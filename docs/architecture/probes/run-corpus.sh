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
# baseline and is not 0 now. Any current status other than 0 or 1 (a crash, panic or timeout)
# also counts, whatever the baseline recorded. New files count only if they crash.
#
# Exit codes: 0 = no new failures (or a recording run); 1 = new failures, and the last line is
# "new failures: N"; 2 = an input could not be prepared (no jtv binary; a missing, unreadable
# or empty corpus root; a failed find; or an unreadable or malformed baseline).
set -u
# crashed CODE: true when CODE is neither 0 (success) nor 1 (a handled JtvError).
crashed() { local code="$1"; [[ "$code" -ne 0 && "$code" -ne 1 ]]; }
# split_record LINE: set rec_f, rec_c and rec_r from "<path> <check exit> <run exit>". The two
# exit codes are taken from the end, so a path that contains spaces stays whole.
split_record() { local line="$1"; rec_r="${line##* }"; line="${line% *}"; rec_c="${line##* }"; rec_f="${line% *}"; }
baseline=""
if [[ "${1:-}" == "--baseline" ]]; then baseline="${2:-}"; shift 2 || { echo "--baseline needs a file" >&2; exit 2; }; fi
jtv="${1:-target/debug/jtv}"
[[ -x "$jtv" ]] || { echo "build first: cargo build -p jtv-cli" >&2; exit 2; }
if [[ -n "$baseline" ]]; then
  [[ -f "$baseline" && -r "$baseline" && -s "$baseline" ]] || { echo "unreadable or empty baseline $baseline" >&2; exit 2; }
  if grep -qvE '^.+ [0-9]+ [0-9]+$' "$baseline"; then echo "malformed baseline $baseline" >&2; exit 2; fi
fi
# Every root must exist and contribute at least one program, and find must succeed: a missing,
# unreadable or empty section would otherwise vanish from the baseline for good.
roots=(examples conformance/valid crates/jtv-core/stdlib)
files=()
for root in "${roots[@]}"; do
  [[ -d "$root" && -r "$root" ]] || { echo "missing or unreadable corpus root $root (run from the repository root)" >&2; exit 2; }
  listing="$(find "$root" -name '*.jtv' -type f)" || { echo "find failed under $root" >&2; exit 2; }
  [[ -n "$listing" ]] || { echo "empty corpus root $root" >&2; exit 2; }
  mapfile -t -O "${#files[@]}" files <<<"$listing"
done
mapfile -t files < <(printf '%s\n' "${files[@]}" | LC_ALL=C sort)
# The CLI exits 1 for a file it cannot read, which would look like a failing program.
for f in "${files[@]}"; do [[ -f "$f" && -r "$f" ]] || { echo "unreadable corpus file $f" >&2; exit 2; }; done
current="$(for f in "${files[@]}"; do
  timeout 30 "$jtv" check "$f" >/dev/null 2>&1; c=$?
  timeout 30 "$jtv" run "$f" >/dev/null 2>&1; r=$?
  printf '%s %s %s\n' "$f" "$c" "$r"
done)"
if [[ -z "$baseline" ]]; then printf '%s\n' "$current"; exit 0; fi
# Index both lists by the full path, so paths are compared as whole strings.
declare -A cur_c cur_r in_base
while IFS= read -r line; do split_record "$line"; cur_c[$rec_f]="$rec_c"; cur_r[$rec_f]="$rec_r"; done <<<"$current"
new=0
while IFS= read -r line; do
  split_record "$line"; f="$rec_f"; bc="$rec_c"; br="$rec_r"; in_base[$f]=1
  if [[ -z "${cur_c[$f]+set}" ]]; then echo "NEW FAILURE $f: missing (baseline check $bc, run $br)"; new=$((new + 1)); continue; fi
  c="${cur_c[$f]}"; r="${cur_r[$f]}"
  # Crashes are counted separately below, for every file, so skip them here.
  if crashed "$c" || crashed "$r"; then continue; fi
  if [[ "$bc" -eq 0 && "$c" -ne 0 ]] || [[ "$br" -eq 0 && "$r" -ne 0 ]]; then
    echo "NEW FAILURE $f: check $bc->$c, run $br->$r"; new=$((new + 1))
  fi
done < "$baseline"
for f in "${files[@]}"; do
  c="${cur_c[$f]}"; r="${cur_r[$f]}"
  # Any status other than 0 or 1 is a crash (101 panic, 134 abort, 124 timeout, ...) and fails,
  # whatever the baseline recorded, including for a program that already failed with 1 or is new.
  if crashed "$c" || crashed "$r"; then echo "NEW FAILURE $f: crash (check $c, run $r)"; new=$((new + 1)); fi
  [[ -n "${in_base[$f]+set}" ]] || echo "new file (not counted unless it crashes): $f"
done
echo "new failures: $new"
[[ "$new" -eq 0 ]]
