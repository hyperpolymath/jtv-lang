// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Julia the Viper - Parser Fuzzer
// Feeds random UTF-8 through the pest parser to detect panics, hangs,
// and memory-safety issues. The parser must NEVER panic on invalid
// input; it should always return a clean error.

#![no_main]
use libfuzzer_sys::fuzz_target;
use jtv_core::parse_program;

fuzz_target!(|data: &[u8]| {
    // Reject oversized inputs to keep fuzzing fast
    if data.is_empty() || data.len() > 50_000 {
        return;
    }

    // Only test valid UTF-8 (pest operates on &str, not &[u8])
    if let Ok(text) = std::str::from_utf8(data) {
        // The parser must never panic. It may return Ok or Err, but
        // both outcomes are acceptable. A panic is a bug.
        let _ = parse_program(text);
    }

    // Also test with lossy UTF-8 conversion to exercise edge cases
    // around replacement characters in identifiers and strings.
    if data.len() < 10_000 {
        let lossy = String::from_utf8_lossy(data);
        let _ = parse_program(&lossy);
    }
});
