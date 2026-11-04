#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# Re-run the 2026-11-04 hole probes (see ../LANGUAGE-SYSTEM-ARCHITECTURE.adoc,
# hole register H1-H19). Prints `jtv check` and `jtv run` exit codes per probe,
# then a nested-parenthesis parse-time sweep (H13). Read-only: writes only to a
# temporary directory. Usage: docs/architecture/probes/run-probes.sh [path/to/jtv]
set -u
here="$(cd "$(dirname "$0")" && pwd)"
jtv="${1:-$here/../../../target/debug/jtv}"
[ -x "$jtv" ] || { echo "build first: cargo build -p jtv-cli" >&2; exit 2; }
printf '%-20s %6s %6s\n' probe check run
for p in "$here"/*.jtv; do
  n="$(basename "$p" .jtv)"
  timeout 30 "$jtv" check "$p" >/dev/null 2>&1; c=$?
  timeout 30 "$jtv" run "$p" >/dev/null 2>&1; r=$?
  printf '%-20s %6s %6s\n' "$n" "$c" "$r"
done
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
echo; echo "H13 parse sweep (depth, check exit, ms):"
for d in ${NEST_DEPTHS:-8 12 16 18}; do
  open="$(printf '(%.0s' $(seq 1 "$d"))"; close="$(printf ')%.0s' $(seq 1 "$d"))"
  printf 'x = %s1%s\nprint(x)\n' "$open" "$close" > "$tmp/n.jtv"
  s=$(date +%s%N); timeout 120 "$jtv" check "$tmp/n.jtv" >/dev/null 2>&1; e=$?
  echo "  $d $e $(( ($(date +%s%N) - s) / 1000000 ))"
done
