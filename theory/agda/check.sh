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

# Resolve this script's directory *before* changing into it: invoking the
# script from the repository root (`theory/agda/check.sh`) and then asking for
# `dirname "$0"` again would look for `theory/agda/theory/agda`. The failure
# was silent — the script has no `set -e` — and the library registry then got a
# nonexistent path, so AGDA_STDLIB runs from the repo root failed confusingly.
HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE"

# The receipt is a *file* as well as terminal output: CI uploads
# _build/check.log as the run's artifact, and the ledger cites exactly that
# path. Create it here, so the artifact step can never point at a directory
# nothing writes (it once did — the receipt was silently absent).
BUILD="_build"
mkdir -p "$BUILD"
exec > >(tee "$BUILD/check.log") 2>&1

AGDA="${AGDA:-agda}"
# Default to *every* module under src/Jtv, not a hard-coded list: the gate is
# triggered by any change under theory/agda/**, so a newly added module must be
# compiled by it. (A fixed list would let CI go green while the new module was
# never checked — the escape-hatch grep catches markers, not type errors.)
# Discovery is *recursive* and covers every namespace: a module added at
# src/Jtv/Algebra/Group.agda is triggered by the same workflow, so it must be
# compiled and scanned too. Paths are made relative to src/ without the .agda
# suffix (Agda wants `src/Jtv/Algebra/Group.agda`; callers may pass either a
# bare name like `Core` or a namespaced one like `Jtv/Core`).
MODULES=("$@")
if [ ${#MODULES[@]} -eq 0 ]; then
  mapfile -t MODULES < <(find src -name '*.agda' -print 2>/dev/null \
    | sed -e 's|^src/||' -e 's|\.agda$||' | sort)
  if [ ${#MODULES[@]} -eq 0 ]; then
    echo "no modules found under src/" >&2; exit 2
  fi
fi

# Resolve a module name to a source path: bare names live in Jtv/.
module_path() {
  case "$1" in
    */*) printf 'src/%s.agda' "$1" ;;
    *)   printf 'src/Jtv/%s.agda' "$1" ;;
  esac
}

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
  if out=$("$AGDA" "${FLAGS[@]}" ${LIBFLAGS[@]+"${LIBFLAGS[@]}"} "$(module_path "$m")" 2>&1); then
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
  if grep -rnE --include='*.agda' '\{[!][!]\}|\{-# (NON_)?TERMINATING|postulate|primTrustMe' src >/dev/null 2>&1; then
    echo "ESCAPE HATCH FOUND — this receipt is not evidence:"
    grep -rnE --include='*.agda' '\{[!][!]\}|postulate' src
    exit 1
  fi
  echo "hole/postulate scan: clean"
else
  echo "result: FAILED (see above)"
fi
exit $fail
