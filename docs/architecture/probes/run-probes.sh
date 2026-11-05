#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# Re-run the 2026-11-04 hole probes (see ../LANGUAGE-SYSTEM-ARCHITECTURE.adoc,
# hole register H1-H19). Prints `jtv check` and `jtv run` exit codes per probe,
# then a nested-parenthesis parse-time sweep (H13). Read-only: writes only to a
# temporary directory. Usage: docs/architecture/probes/run-probes.sh [--gate] [path/to/jtv]
#
# Report mode (default, Wave 0 baseline) exits 0 even when holes reproduce, which is expected;
# both modes exit 2 if any input cannot be prepared (a missing, unlisted or unreadable
# probe, failed mktemp or sweep write). --gate mode also exits 1 if any probe or sweep step ends with anything other
# than 0 (success) or 1 (a handled JtvError): a timeout (124), a Rust panic (101), a
# host abort (134) or any other status counts as a crash, so none can pass Gate 1.
# Whether each probe's behaviour is correct is judged per file by the Gate 1
# falsifiers, not by this script.
set -u
gate=0
if [[ "${1:-}" == "--gate" ]]; then gate=1; shift; fi
here="$(cd "$(dirname "$0")" && pwd)"
jtv="${1:-$here/../../../target/debug/jtv}"
bad=0
prep_bad=0
# crashed CODE: true when CODE is neither 0 (success) nor 1 (handled JtvError).
crashed() { local code="$1"; [[ "$code" -ne 0 && "$code" -ne 1 ]]; }
# prep_failed WHAT: an input could not be prepared, so its check never ran; count it.
prep_failed() { local what="$1"; echo "  preparation failed: $what"; bad=$((bad + 1)); prep_bad=$((prep_bad + 1)); }
[[ -x "$jtv" ]] || { echo "build first: cargo build -p jtv-cli" >&2; exit 2; }
# Every committed probe is listed by name: a probe deleted or renamed during integration
# fails preparation instead of silently dropping out, and an unlisted probe must be added here.
expected=(any_rev blowup dyn fn_rev kw mutual ok pure_loop_in_data rec_total rev_if_overwrite
  rev_if_tok rev_if_tokenless shadow shadow_loop str_rev sym_rev3)
probes=()
for n in "${expected[@]}"; do probes+=("$here/$n.jtv"); done
shopt -s nullglob
for p in "$here"/*.jtv; do
  n="$(basename "$p" .jtv)"
  [[ " ${expected[*]} " == *" $n "* ]] || prep_failed "unlisted probe $p (add it to expected)"
done
printf '%-20s %6s %6s  %s\n' probe check run 'output (all lines, joined with |)'
for p in "${probes[@]}"; do
  n="$(basename "$p" .jtv)"
  # The CLI also exits 1 when it cannot read a file, so check readability first.
  [[ -f "$p" && -r "$p" ]] || { prep_failed "missing or unreadable probe $p"; continue; }
  timeout 30 "$jtv" check "$p" >/dev/null 2>&1; c=$?
  out="$(timeout 30 "$jtv" run "$p" 2>&1)"; r=$?
  joined="$(printf '%s\n' "$out" | head -n 8 | paste -sd '|' | cut -c1-140)"
  printf '%-20s %6s %6s  %s\n' "$n" "$c" "$r" "$joined"
  if crashed "$c" || crashed "$r"; then bad=$((bad + 1)); fi
done
echo; echo "H13 parse sweep (depth, check exit, ms):"
if tmp="$(mktemp -d)" && [[ -d "$tmp" ]]; then
  trap 'rm -rf "$tmp"' EXIT
else
  prep_failed "mktemp -d"; tmp=""
fi
for d in ${NEST_DEPTHS:-8 12 16 18}; do
  [[ -n "$tmp" ]] || { prep_failed "sweep depth $d (no temp dir)"; continue; }
  open="$(printf '(%.0s' $(seq 1 "$d"))"; close="$(printf ')%.0s' $(seq 1 "$d"))"
  printf 'x = %s1%s\nprint(x)\n' "$open" "$close" > "$tmp/n.jtv" && [[ -s "$tmp/n.jtv" ]] \
    || { prep_failed "sweep file for depth $d"; continue; }
  s=$(date +%s%N); timeout 120 "$jtv" check "$tmp/n.jtv" >/dev/null 2>&1; e=$?
  echo "  $d $e $(( ($(date +%s%N) - s) / 1000000 ))"
  if crashed "$e"; then bad=$((bad + 1)); fi
done
echo; echo "hung, crashed or unprepared steps: $bad (unprepared: $prep_bad)"
# Missing evidence fails in both modes: a baseline built on probes that never ran is not a baseline.
if [[ "$prep_bad" -ne 0 ]]; then exit 2; fi
if [[ "$gate" -eq 1 && "$bad" -ne 0 ]]; then exit 1; fi
exit 0
