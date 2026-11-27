// SPDX-License-Identifier: MPL-2.0
//
// The shipped basic examples that type-check must keep type-checking.
//
// No other test runs the checker over the files under `examples/`, so a change
// to a typing rule could reject them unnoticed. That happened once: an
// automated fix made Data calls require a callee declared `@total` and made
// `return` match the declared type exactly, which rejected the two function
// examples below while every test still passed.
//
// This pins only the examples that pass today. The other shipped examples fail
// `jtv check` for reasons the hole register tracks; they are not listed here
// and this test says nothing about them.

use jtv_core::{parser::parse_program, TypeChecker};

const EXAMPLES: [(&str, &str); 3] = [
    (
        "examples/basic/01_hello_addition.jtv",
        include_str!("../../../examples/basic/01_hello_addition.jtv"),
    ),
    (
        "examples/basic/03_functions.jtv",
        include_str!("../../../examples/basic/03_functions.jtv"),
    ),
    (
        "examples/basic/05_conditionals.jtv",
        include_str!("../../../examples/basic/05_conditionals.jtv"),
    ),
];

#[test]
fn shipped_basic_examples_type_check() {
    for (path, src) in EXAMPLES {
        let program = parse_program(src).unwrap_or_else(|e| panic!("{path} does not parse: {e}"));
        TypeChecker::new()
            .check_program(&program)
            .unwrap_or_else(|e| panic!("{path} does not type-check: {e}"));
    }
}
