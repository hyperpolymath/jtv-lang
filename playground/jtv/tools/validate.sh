#!/bin/sh
# SPDX-License-Identifier: MPL-2.0
# validate.sh - JTV corpus validator (f0 stub implementation)
#
# This is a placeholder validator for the f0 milestone.
# It performs basic syntax checks on JTV files.
# A proper parser will be implemented in f1.

set -e

CORPUS_DIR="${1:-jtv/tests/corpus}"
VALID_DIR="$CORPUS_DIR/valid"
INVALID_DIR="$CORPUS_DIR/invalid"

valid_count=0
valid_passed=0
invalid_count=0
invalid_detected=0

echo "JTV Corpus Validator (f0 stub)"
echo "=============================="
echo ""

# Validate valid samples (should pass)
echo "Checking valid samples..."
for file in "$VALID_DIR"/*.jtv; do
    [ -f "$file" ] || continue
    valid_count=$((valid_count + 1))
    filename=$(basename "$file")

    # f0 validation: check for required elements
    has_module=$(grep -c "^module " "$file" 2>/dev/null || echo "0")
    has_fn=$(grep -c "^fn " "$file" 2>/dev/null || echo "0")

    if [ "$has_module" -ge 1 ] && [ "$has_fn" -ge 1 ]; then
        echo "  PASS: $filename"
        valid_passed=$((valid_passed + 1))
    else
        echo "  FAIL: $filename (missing module or fn declaration)"
    fi
done

echo ""

# Validate invalid samples (should fail)
echo "Checking invalid samples (expecting errors)..."
for file in "$INVALID_DIR"/*.jtv; do
    [ -f "$file" ] || continue
    invalid_count=$((invalid_count + 1))
    filename=$(basename "$file")

    # f0 validation: check for missing elements or error markers
    has_error_comment=$(grep -c "ERROR:" "$file" 2>/dev/null || echo "0")
    has_module=$(grep -c "^module " "$file" 2>/dev/null || echo "0")

    # For invalid files, we expect either missing module OR an ERROR comment
    if [ "$has_error_comment" -ge 1 ] || [ "$has_module" -eq 0 ]; then
        echo "  DETECTED: $filename (correctly identified as invalid)"
        invalid_detected=$((invalid_detected + 1))
    else
        echo "  MISSED: $filename (failed to detect error)"
    fi
done

echo ""
echo "=============================="
echo "Results:"
echo "  Valid samples:   $valid_passed/$valid_count passed"
echo "  Invalid samples: $invalid_detected/$invalid_count detected"
echo ""

total_samples=$((valid_count + invalid_count))
if [ "$total_samples" -lt 5 ]; then
    echo "WARNING: Need at least 5 samples (have $total_samples)"
    exit 1
fi

if [ "$valid_passed" -lt 3 ]; then
    echo "WARNING: Need at least 3 valid samples passing (have $valid_passed)"
    exit 1
fi

if [ "$invalid_detected" -lt 2 ]; then
    echo "WARNING: Need at least 2 invalid samples detected (have $invalid_detected)"
    exit 1
fi

if [ "$valid_passed" -eq "$valid_count" ] && [ "$invalid_detected" -eq "$invalid_count" ]; then
    echo "SUCCESS: All tests passed!"
    exit 0
else
    echo "PARTIAL: Some tests failed"
    exit 1
fi
