#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# Record, or compare against a baseline, the `jtv check` and `jtv run` exit codes of every
# program in the regression corpus: examples/**, conformance/valid/* and
# crates/jtv-core/stdlib/*. Read-only. Run from the repository root (it reads the corpus
# relative to the current directory).
#
# Usage: docs/architecture/probes/run-corpus.sh [--baseline FILE] [path/to/jtv]
#
# Without --baseline it prints one line per file, "<path> <check exit> <run exit>", sorted;
# Wave 0 saves that as the baseline. With --baseline it reruns the corpus and counts a new
# failure for every baseline file that is now missing, or whose check or run exit was 0 in the
# baseline and is not 0 now. Files new since the baseline are reported but never count.
#
# Exit codes: 0 = no new failures (or a recording run); 1 = new failures, and the last line is
# "new failures: N"; 2 = an input could not be prepared (no jtv binary, an empty corpus, or an
# unreadable or malformed baseline).
set -u
baseline=""
if [[ "${1:-}" == "--baseline" ]]; then baseline="${2:-}"; shift 2 || { echo "--baseline needs a file" >&2; exit 2; }; fi
jtv="${1:-target/debug/jtv}"
[[ -x "$jtv" ]] || { echo "build first: cargo build -p jtv-cli" >&2; exit 2; }
if [[ -n "$baseline" ]]; then
  [[ -f "$baseline" && -r "$baseline" && -s "$baseline" ]] || { echo "unreadable or empty baseline $baseline" >&2; exit 2; }
  if grep -qvE '^[^ ]+ [0-9]+ [0-9]+$' "$baseline"; then echo "malformed baseline $baseline" >&2; exit 2; fi
fi
mapfile -t files < <(find examples conformance/valid crates/jtv-core/stdlib -name '*.jtv' -type f 2>/dev/null | LC_ALL=C sort)
[[ "${#files[@]}" -gt 0 ]] || { echo "empty corpus (run from the repository root)" >&2; exit 2; }
current="$(for f in "${files[@]}"; do
  timeout 30 "$jtv" check "$f" >/dev/null 2>&1; c=$?
  timeout 30 "$jtv" run "$f" >/dev/null 2>&1; r=$?
  printf '%s %s %s\n' "$f" "$c" "$r"
done)"
if [[ -z "$baseline" ]]; then printf '%s\n' "$current"; exit 0; fi
new=0
while read -r f bc br; do
  line="$(printf '%s\n' "$current" | awk -v f="$f" '$1 == f')"
  if [[ -z "$line" ]]; then echo "NEW FAILURE $f: missing (baseline check $bc, run $br)"; new=$((new + 1)); continue; fi
  read -r _ c r <<<"$line"
  if [[ "$bc" -eq 0 && "$c" -ne 0 ]] || [[ "$br" -eq 0 && "$r" -ne 0 ]]; then
    echo "NEW FAILURE $f: check $bc->$c, run $br->$r"; new=$((new + 1))
  fi
done < "$baseline"
while read -r f _ _; do
  awk -v f="$f" '$1 == f { found = 1 } END { exit !found }' "$baseline" || echo "new file (not counted): $f"
done <<<"$current"
echo "new failures: $new"
[[ "$new" -eq 0 ]]
