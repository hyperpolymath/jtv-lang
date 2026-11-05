#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# List every workspace test as "<binary>::<test name>", sorted, or check that every test in a
# baseline list still exists. A count alone cannot show that a pre-Wave-1 test survived: new
# tests can hide a deleted one. Run from the repository root.
#
# Usage: docs/architecture/probes/list-tests.sh [--baseline FILE]
#
# Without --baseline it prints the list (Wave 0 saves it as receipts/tests-w0.txt). With
# --baseline it prints each baseline test that is now missing and a final "missing tests: N".
#
# Exit codes: 0 = listed, or no baseline test missing; 1 = at least one baseline test missing;
# 2 = an input could not be prepared (cargo failed, no tests listed, or an unreadable, empty or
# malformed baseline).
set -u
baseline=""
if [[ "${1:-}" == "--baseline" ]]; then baseline="${2:-}"; shift 2 || { echo "--baseline needs a file" >&2; exit 2; }; fi
if [[ -n "$baseline" ]]; then
  [[ -f "$baseline" && -r "$baseline" && -s "$baseline" ]] || { echo "unreadable or empty baseline $baseline" >&2; exit 2; }
  if grep -qvE '^[^[:space:]:]+::.+$' "$baseline"; then echo "malformed baseline $baseline" >&2; exit 2; fi
fi
# cargo prints "Running <src> (<path>)" and "Doc-tests <crate>" on stderr before each binary's
# listing, so merging the streams keeps every test under the binary it belongs to.
raw="$(cargo test --workspace -- --list 2>&1)" || { echo "cargo test --list failed" >&2; exit 2; }
current="$(printf '%s\n' "$raw" | awk '
  /^[[:space:]]+Running / { p = $NF; gsub(/[()]/, "", p); sub(/.*\//, "", p); sub(/-[0-9a-f]+$/, "", p); bin = p; next }
  /^[[:space:]]+Doc-tests / { bin = "doc-" $2; next }
  / (test|bench)$/ && bin != "" { name = $0; sub(/: (test|bench)$/, "", name); print bin "::" name }
' | LC_ALL=C sort -u)"
[[ -n "$current" ]] || { echo "no tests listed" >&2; exit 2; }
if [[ -z "$baseline" ]]; then printf '%s\n' "$current"; exit 0; fi
missing="$(LC_ALL=C comm -23 <(LC_ALL=C sort -u "$baseline") <(printf '%s\n' "$current"))"
n=0
if [[ -n "$missing" ]]; then
  while read -r t; do echo "MISSING $t"; n=$((n + 1)); done <<<"$missing"
fi
echo "missing tests: $n"
[[ "$n" -eq 0 ]]
