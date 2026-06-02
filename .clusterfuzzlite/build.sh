#!/bin/bash -eu

cd "$SRC"/project

SANITIZER="${SANITIZER:-address}"

if [ "$SANITIZER" = "address" ]; then
  cargo +nightly fuzz build --release
elif [ "$SANITIZER" = "undefined" ]; then
  # Rust has no UBSan equivalent via -Z sanitizer.
  # Build without any sanitizer so bad_build_check sees no ASan instrumentation.
  cargo +nightly fuzz build --sanitizer none --release
else
  cargo +nightly fuzz build --release
fi

find fuzz/target -maxdepth 4 -name 'fuzz_*' -executable \
  -not -name '*.d' -exec cp {} "$OUT"/ \;
