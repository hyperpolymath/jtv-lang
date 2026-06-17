// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! JtV v2 reversibility Phase 1 tests.
//!
//! Verifies that `reverse { x += v }` IS subtraction (x = x - v), per the
//! canonical design in `docs/language/DESIGN-JTV-V2-REVERSIBILITY.md`.
//!
//! Subtraction is not a grammar primitive in JtV v2.  It arises from
//! reversing addition.  `reverse { x += v }` does NOT do a forward pass —
//! it applies the INVERSE of each operation in reverse declaration order.
//!
//! RV1 — single add: reverse { x += 5 } → x = x - 5
//! RV2 — single sub: reverse { x -= 3 } → x = x + 3
//! RV3 — chain:     reverse { x += 5; y += 3 } → y -= 3 first, then x -= 5
//! RV4 — cross-var: reverse { x += y } → x = x - y
//! RV5 — CNO via execute_and_reverse: net effect is identity
//! RV6 — full-program parse + run with reverse { }
//! RV7 — reverse is left-inverse of forward: (forward; reverse) = identity

use jtv_core::{
    ast::{DataExpr, Number, ReverseBlock, ReversibleStmt},
    number::Value,
    parser::parse_program,
    reversible::ReversibleInterpreter,
    Interpreter,
};

// ── RV1: reverse add is subtract ────────────────────────────────────────────

#[test]
fn rv1_reverse_add_is_subtract() {
    let mut interp = Interpreter::new();
    let src = r#"
x = 10
reverse { x += 5 }
"#;
    let prog = parse_program(src).expect("should parse");
    interp.run(&prog).expect("should run");

    let x = interp
        .get_variables()
        .into_iter()
        .find(|(k, _)| k == "x")
        .map(|(_, v)| v);
    assert_eq!(
        x,
        Some(Value::Int(5)),
        "reverse {{ x += 5 }} with x=10 should give x=5 (subtraction)"
    );
}

// ── RV2: reverse sub is add ──────────────────────────────────────────────────

#[test]
fn rv2_reverse_sub_is_add() {
    let mut interp = Interpreter::new();
    let src = r#"
x = 10
reverse { x -= 3 }
"#;
    let prog = parse_program(src).expect("should parse");
    interp.run(&prog).expect("should run");

    let x = interp
        .get_variables()
        .into_iter()
        .find(|(k, _)| k == "x")
        .map(|(_, v)| v);
    assert_eq!(
        x,
        Some(Value::Int(13)),
        "reverse {{ x -= 3 }} with x=10 should give x=13 (inverse of subtraction is addition)"
    );
}

// ── RV3: multi-op chain inverts in reverse order ─────────────────────────────

#[test]
fn rv3_chain_inverts_in_reverse_declaration_order() {
    // reverse { x += 5 ; y += 3 }
    // Should apply: y -= 3 first, then x -= 5
    // Both are independent, so order doesn't matter here; test values confirm semantics.
    let mut interp = Interpreter::new();
    let src = r#"
x = 10
y = 20
reverse { x += 5 y += 3 }
"#;
    let prog = parse_program(src).expect("should parse");
    interp.run(&prog).expect("should run");

    let vars: std::collections::HashMap<String, Value> =
        interp.get_variables().into_iter().collect();
    assert_eq!(vars.get("x"), Some(&Value::Int(5)), "x should be 10-5=5");
    assert_eq!(vars.get("y"), Some(&Value::Int(17)), "y should be 20-3=17");
}

// ── RV4: cross-variable: reverse { x += y } → x = x - y ────────────────────

#[test]
fn rv4_cross_variable_reverse() {
    let mut interp = Interpreter::new();
    let src = r#"
x = 10
y = 3
reverse { x += y }
"#;
    let prog = parse_program(src).expect("should parse");
    interp.run(&prog).expect("should run");

    let x = interp
        .get_variables()
        .into_iter()
        .find(|(k, _)| k == "x")
        .map(|(_, v)| v);
    assert_eq!(
        x,
        Some(Value::Int(7)),
        "reverse {{ x += y }} with x=10, y=3 should give x=7"
    );
}

// ── RV5: CNO via execute_and_reverse ────────────────────────────────────────

#[test]
fn rv5_cno_execute_and_reverse_is_identity() {
    let mut interp = ReversibleInterpreter::new();
    interp.set("x".to_string(), Value::Int(42));
    interp.set("y".to_string(), Value::Int(100));

    let block = ReverseBlock {
        body: vec![
            ReversibleStmt::AddAssign("x".to_string(), DataExpr::Number(Number::Int(7))),
            ReversibleStmt::AddAssign("y".to_string(), DataExpr::Number(Number::Int(15))),
        ],
    };

    interp
        .execute_and_reverse(&block)
        .expect("CNO should not fail");

    assert_eq!(
        interp.get("x"),
        Some(&Value::Int(42)),
        "x must be unchanged after CNO"
    );
    assert_eq!(
        interp.get("y"),
        Some(&Value::Int(100)),
        "y must be unchanged after CNO"
    );
}

// ── RV6: full-program parse + run ────────────────────────────────────────────

#[test]
fn rv6_full_program_with_reverse_block() {
    // A meaningful program: accumulate a total, then reverse one step.
    // JtV top-level uses `x = expr` for assignment; `+=` is reverse-block only.
    let src = r#"
total = 100
bonus = 25
total = total + bonus
reverse { total += bonus }
"#;
    // After: total = 100 + 25 = 125; then reverse { total += 25 } → total = 125 - 25 = 100
    let mut interp = Interpreter::new();
    let prog = parse_program(src).expect("should parse");
    interp.run(&prog).expect("should run");

    let vars: std::collections::HashMap<String, Value> =
        interp.get_variables().into_iter().collect();
    assert_eq!(
        vars.get("total"),
        Some(&Value::Int(100)),
        "forward +25 then reverse -25 returns total to 100"
    );
    assert_eq!(vars.get("bonus"), Some(&Value::Int(25)), "bonus unchanged");
}

// ── RV7: forward then reverse is left-inverse ────────────────────────────────

#[test]
fn rv7_forward_then_reverse_is_left_inverse() {
    // `execute_forward` then `execute_inverse` on same block = identity.
    // This is distinct from `execute_and_reverse` (CNO): here we call
    // forward on one interpreter, then inverse on another starting from
    // the FORWARD state — proving they are mutual inverses.
    let block = ReverseBlock {
        body: vec![ReversibleStmt::AddAssign(
            "x".to_string(),
            DataExpr::Number(Number::Int(11)),
        )],
    };

    let mut fwd = ReversibleInterpreter::new();
    fwd.set("x".to_string(), Value::Int(5));
    fwd.execute_forward(&block).expect("forward");
    assert_eq!(
        fwd.get("x"),
        Some(&Value::Int(16)),
        "forward: x = 5 + 11 = 16"
    );

    // Now apply inverse starting from the forward state
    let mut inv = ReversibleInterpreter::with_state(fwd.get_state().clone());
    inv.execute_inverse(&block).expect("inverse");
    assert_eq!(
        inv.get("x"),
        Some(&Value::Int(5)),
        "inverse of forward restores original: 16 - 11 = 5"
    );
}
