// SPDX-License-Identifier: MPL-2.0
// (MPL-2.0 is automatic legal fallback until PMPL is formally recognised)
//
// C2 reversibility tests: reversible { } -> tok / reverse tok / abandon tok
//
// C2-1 — reverse tok restores state (CNO pattern)
// C2-2 — abandon tok commits forward state
// C2-3 — double-consume via reverse errors (linearity)
// C2-4 — reverse after abandon errors (linearity)
// C2-5 — multiple ops in reversible block: all undone by reverse tok

use jtv_core::{number::Value, parser::parse_program, Interpreter};

fn get_var(interp: &Interpreter, name: &str) -> Option<Value> {
    interp
        .get_variables()
        .into_iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v)
}

// C2-1: reversible block binds a token; reverse tok restores state
#[test]
fn c2_1_reverse_token_restores_state() {
    let src = r#"
x = 10
reversible { x += 5 } -> tok
reverse tok
"#;
    let prog = parse_program(src).expect("should parse");
    let mut interp = Interpreter::new();
    interp.run(&prog).expect("should run");
    // forward: x=15; reverse tok: x=10 restored
    assert_eq!(get_var(&interp, "x"), Some(Value::Int(10)));
}

// C2-2: abandon tok commits the forward state; token is consumed
#[test]
fn c2_2_abandon_commits_forward_state() {
    let src = r#"
x = 10
reversible { x += 5 } -> tok
abandon tok
"#;
    let prog = parse_program(src).expect("should parse");
    let mut interp = Interpreter::new();
    interp.run(&prog).expect("should run");
    // abandon keeps x=15
    assert_eq!(get_var(&interp, "x"), Some(Value::Int(15)));
}

// C2-3: token is linear — double-consume via reverse errors
#[test]
fn c2_3_token_linearity_reverse_twice_errors() {
    let src = r#"
x = 0
reversible { x += 1 } -> tok
reverse tok
reverse tok
"#;
    let prog = parse_program(src).expect("should parse");
    let mut interp = Interpreter::new();
    let result = interp.run(&prog);
    assert!(
        result.is_err(),
        "Expected error on double-consume of reversal token"
    );
}

// C2-4: token is linear — reverse after abandon errors
#[test]
fn c2_4_token_linearity_abandon_then_reverse_errors() {
    let src = r#"
x = 0
reversible { x += 1 } -> tok
abandon tok
reverse tok
"#;
    let prog = parse_program(src).expect("should parse");
    let mut interp = Interpreter::new();
    let result = interp.run(&prog);
    assert!(
        result.is_err(),
        "Expected error consuming already-abandoned token"
    );
}

// C2-5: multiple ops in reversible block — all undone by reverse tok
#[test]
fn c2_5_multiple_ops_fully_undone() {
    let src = r#"
x = 100
y = 0
reversible { x += 10 y += 3 x += 2 } -> tok
reverse tok
"#;
    let prog = parse_program(src).expect("should parse");
    let mut interp = Interpreter::new();
    interp.run(&prog).expect("should run");
    // x should be back to 100, y back to 0
    assert_eq!(get_var(&interp, "x"), Some(Value::Int(100)));
    assert_eq!(get_var(&interp, "y"), Some(Value::Int(0)));
}
