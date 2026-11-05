#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# Re-run the 2026-11-04 hole probes (see ../LANGUAGE-SYSTEM-ARCHITECTURE.adoc,
# hole register H1-H19). Prints `jtv check` and `jtv run` exit codes per probe,
# then a nested-parenthesis parse-time sweep (H13). Read-only: writes only to a
# temporary directory. Usage: docs/architecture/probes/run-probes.sh [--gate] [path/to/jtv]
#
# Report mode (default, Wave 0 baseline) always exits 0: the holes are expected to
# reproduce. --gate mode exits 1 if any probe or sweep step ends with anything other
# than 0 (success) or 1 (a handled JtvError): a timeout (124), a Rust panic (101), a
# host abort (134) or any other status counts as a crash, so none can pass Gate 1. Whether each probe's behaviour is correct is judged per file by the Gate 1
# falsifiers, not by this script.
set -u
gate=0
if [[ "${1:-}" == "--gate" ]]; then gate=1; shift; fi
here="$(cd "$(dirname "$0")" && pwd)"
jtv="${1:-$here/../../../target/debug/jtv}"
bad=0
crashed() { local code="$1"; [[ "$code" -ne 0 && "$code" -ne 1 ]]; }
[[ -x "$jtv" ]] || { echo "build first: cargo build -p jtv-cli" >&2; exit 2; }
printf '%-20s %6s %6s  %s\n' probe check run 'output (all lines, joined with |)'
for p in "$here"/*.jtv; do
  n="$(basename "$p" .jtv)"
  timeout 30 "$jtv" check "$p" >/dev/null 2>&1; c=$?
  out="$(timeout 30 "$jtv" run "$p" 2>&1)"; r=$?
  joined="$(printf '%s\n' "$out" | head -n 8 | paste -sd '|' | cut -c1-140)"
  printf '%-20s %6s %6s  %s\n' "$n" "$c" "$r" "$joined"
  if crashed "$c" || crashed "$r"; then bad=$((bad + 1)); fi
done
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
echo; echo "H13 parse sweep (depth, check exit, ms):"
for d in ${NEST_DEPTHS:-8 12 16 18}; do
  open="$(printf '(%.0s' $(seq 1 "$d"))"; close="$(printf ')%.0s' $(seq 1 "$d"))"
  printf 'x = %s1%s\nprint(x)\n' "$open" "$close" > "$tmp/n.jtv"
  s=$(date +%s%N); timeout 120 "$jtv" check "$tmp/n.jtv" >/dev/null 2>&1; e=$?
  echo "  $d $e $(( ($(date +%s%N) - s) / 1000000 ))"
  if crashed "$e"; then bad=$((bad + 1)); fi
done
echo; echo "hung or crashed steps: $bad"
if [[ "$gate" -eq 1 && "$bad" -ne 0 ]]; then exit 1; fi
exit 0
