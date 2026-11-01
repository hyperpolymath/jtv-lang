#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
#
# JtV theory-proof check — the estate's canonical Agda invocation.
#
# Normative sources:
#   hyperpolymath/standards:3-practice/PROOF-POLICY.adoc §2.1 (invocation)
#   hyperpolymath/standards:3-practice/PROOF-POLICY.adoc §6   (toolchains)
#   hyperpolymath/mathesis:estate-mathematical-architecture §6 (receipts)
#
# A run of this script is a *receipt*: it prints the exact tool versions,
# the invocation, and a per-theorem result. Record the receipt with the
# commit it describes; mathesis tracks the commit hash, not the claim.
#
# Usage:  ./check.sh [module ...]      (default: all modules)
# Env:    AGDA         agda binary           (default: agda)
#         AGDA_STDLIB  path to a .agda-lib (optional). When set it is wired
#                      into Agda via an explicit --library-file, so no
#                      ~/.agda/libraries registration is needed. When unset,
#                      Agda uses the user's own library configuration.
set -uo pipefail

cd "$(dirname "$0")"

# The receipt is a *file* as well as terminal output: CI uploads
# _build/check.log as the run's artifact, and the ledger cites exactly that
# path. Create it here, so the artifact step can never point at a directory
# nothing writes (it once did — the receipt was silently absent).
BUILD="_build"
mkdir -p "$BUILD"
exec > >(tee "$BUILD/check.log") 2>&1

AGDA="${AGDA:-agda}"
MODULES=("$@")
if [ ${#MODULES[@]} -eq 0 ]; then
  MODULES=(Core Reversal Echo Conditional)
fi

# --safe is the checker's safe mode; --without-K and --cubical-compatible
# are the policy's flags; the warning switch silences the cubical
# indexed-match note the stdlib itself sets in its .agda-lib.
FLAGS=(--safe --without-K --cubical-compatible --warning=noUnsupportedIndexedMatch)

# Wire AGDA_STDLIB into Agda for real. NOTE: Agda's --library-file takes a
# file that *lists* .agda-lib paths (the same shape as ~/.agda/libraries),
# not a single .agda-lib. So we write a temporary libraries file holding the
# given stdlib plus this project's own jtv-theory.agda-lib, and point Agda at
# it with --no-default-libraries.
LIBFLAGS=()
TMPLIBS=""
cleanup() { [ -n "$TMPLIBS" ] && rm -f "$TMPLIBS"; }
trap cleanup EXIT
if [ -n "${AGDA_STDLIB:-}" ]; then
  if [ ! -f "$AGDA_STDLIB" ]; then
    echo "AGDA_STDLIB=$AGDA_STDLIB does not exist" >&2; exit 2
  fi
  HERE="$(cd "$(dirname "$0")" && pwd)"
  TMPLIBS="$(mktemp)"
  printf '%s\n%s\n' "$AGDA_STDLIB" "$HERE/jtv-theory.agda-lib" > "$TMPLIBS"
  LIBFLAGS=(--library-file "$TMPLIBS" --no-default-libraries)
fi

echo "# JtV theory proofs — Agda"
echo "agda:            $("$AGDA" --version 2>&1 | head -1)"
echo "stdlib:          ${AGDA_STDLIB:-per ~/.agda/defaults}"
echo "invocation:      $AGDA ${FLAGS[*]} ${LIBFLAGS[*]:-} <module>.agda"
echo "commit:          $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
echo "date:            $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo

fail=0
for m in "${MODULES[@]}"; do
  printf '%-14s ' "$m"
  if out=$("$AGDA" "${FLAGS[@]}" ${LIBFLAGS[@]+"${LIBFLAGS[@]}"} "src/Jtv/$m.agda" 2>&1); then
    echo "TYPECHECKED"
  else
    echo "FAILED"
    echo "$out" | grep -v '^ *Checking' | head -20
    fail=1
  fi
done

echo
if [ $fail -eq 0 ]; then
  echo "result: all modules typecheck (no holes: see the grep below)"
  # An escape hatch would make a "green" run meaningless (PROOF-POLICY §5).
  if grep -rnE '\{[!][!]\}|\{-# (NON_)?TERMINATING|postulate|primTrustMe' src/Jtv/*.agda >/dev/null 2>&1; then
    echo "ESCAPE HATCH FOUND — this receipt is not evidence:"; grep -rnE '\{[!][!]\}|postulate' src/Jtv/*.agda
    exit 1
  fi
  echo "hole/postulate scan: clean"
else
  echo "result: FAILED (see above)"
fi
exit $fail
