#!/bin/sh
# SPDX-License-Identifier: MPL-2.0
# demo.sh - JTV language demonstration (f0)
#
# Demonstrates JTV language features without requiring network access.

set -e

SAMPLES_DIR="${1:-jtv/samples}"

echo "=============================================="
echo "  Julia-the-Viper (JTV) Language Demo"
echo "  Harvard Architecture Systems Programming"
echo "=============================================="
echo ""
echo "JTV is designed for embedded systems with separate"
echo "code (flash) and data (RAM) memory spaces."
echo ""

# Show hello world example
echo "--- Example 1: Hello World ---"
echo ""
if [ -f "$SAMPLES_DIR/hello.jtv" ]; then
    cat "$SAMPLES_DIR/hello.jtv"
else
    echo "(sample file not found)"
fi
echo ""

echo "--- Key JTV Features ---"
echo ""
echo "1. Harvard Architecture Memory Spaces:"
echo "   @code  - Flash/ROM (program memory)"
echo "   @data  - RAM (data memory)"
echo "   @io    - Memory-mapped I/O registers"
echo ""

echo "2. Resource Constraints:"
echo "   #[max_stack(N)]  - Compile-time stack verification"
echo "   #[no_heap]       - Static allocation only"
echo ""

echo "3. Interrupt Handlers:"
echo "   #[interrupt(VECTOR)]  - ISR declaration"
echo ""

# Show blink example
echo "--- Example 2: LED Blink with Interrupts ---"
echo ""
if [ -f "$SAMPLES_DIR/blink.jtv" ]; then
    head -30 "$SAMPLES_DIR/blink.jtv"
    echo "   ... (see full file for complete example)"
else
    echo "(sample file not found)"
fi
echo ""

echo "=============================================="
echo "  Demo complete (offline, no network required)"
echo "=============================================="
