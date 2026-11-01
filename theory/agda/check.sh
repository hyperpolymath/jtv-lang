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
# Env:    AGDA       agda binary           (default: agda)
#         AGDA_STDLIB  path to a .agda-lib (optional; else the user's
#                      ~/.agda/libraries must register the stdlib)
set -uo pipefail

cd "$(dirname "$0")"
AGDA="${AGDA:-agda}"
MODULES=("$@")
if [ ${#MODULES[@]} -eq 0 ]; then
  MODULES=(Core Reversal Echo Conditional)
fi

# --safe is the checker's safe mode; --without-K and --cubical-compatible
# are the policy's flags; the warning switch silences the cubical
# indexed-match note the stdlib itself sets in its .agda-lib.
FLAGS=(--safe --without-K --cubical-compatible --warning=noUnsupportedIndexedMatch)

echo "# JtV theory proofs — Agda"
echo "agda:            $("$AGDA" --version 2>&1 | head -1)"
echo "stdlib:          $(awk '/^name:/{print $2}' "${AGDA_STDLIB:-/dev/null}" 2>/dev/null || echo 'per ~/.agda/defaults')"
echo "invocation:      $AGDA ${FLAGS[*]} <module>.agda"
echo "commit:          $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
echo "date:            $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo

fail=0
for m in "${MODULES[@]}"; do
  printf '%-14s ' "$m"
  if out=$("$AGDA" "${FLAGS[@]}" "src/Jtv/$m.agda" 2>&1); then
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
