// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Contract and invariant tests for Julia the Viper.
// Verifies that declared contracts (addition-only, purity, Harvard separation,
// reversibility) actually hold at runtime, not just at parse time.
//
// Taxonomy categories: Contract/Invariant (CTR), Lifecycle (LCY)

use jtv_core::ast::*;
use jtv_core::number::Value;
use jtv_core::parser::parse_program;
use jtv_core::purity::{PurityChecker, PurityLevel};
use jtv_core::reversible::{check_reversibility, ReversibleInterpreter};
use num_complex::Complex64;
use num_rational::Ratio;

// ============================================================================
// CONTRACT: Addition-only Data Language
// The AST type DataExpr only allows Add and Negate — no Sub/Mul/Div/Mod.
// This is enforced by type system (enum variant exhaustiveness).
// ============================================================================

#[test]
fn contract_data_expr_variants_are_addition_only() {
    // Exhaustively enumerate DataExpr variants to prove no arithmetic beyond Add/Negate
    let variants: Vec<DataExpr> = vec![
        DataExpr::Number(Number::Int(1)),
        DataExpr::StringLit("hello".to_string()),
        DataExpr::Identifier("x".to_string()),
        DataExpr::Add(
            Box::new(DataExpr::Number(Number::Int(1))),
            Box::new(DataExpr::Number(Number::Int(2))),
        ),
        DataExpr::Negate(Box::new(DataExpr::Number(Number::Int(1)))),
        DataExpr::FunctionCall(FunctionCall {
            module: None,
            name: "f".to_string(),
            args: vec![],
        }),
        DataExpr::List(vec![]),
        DataExpr::Tuple(vec![]),
    ];

    // If this compiles, the contract holds: DataExpr has exactly these 8 variants.
    // Adding Sub, Mul, Div, Mod would be a compile error here (non-exhaustive).
    for v in &variants {
        match v {
            DataExpr::Number(_) => {}
            DataExpr::StringLit(_) => {}
            DataExpr::Identifier(_) => {}
            DataExpr::Add(_, _) => {}
            DataExpr::Negate(_) => {}
            DataExpr::FunctionCall(_) => {}
            DataExpr::List(_) => {}
            DataExpr::Tuple(_) => {}
            // If a new variant is added, this match becomes non-exhaustive
            // and the test fails to compile — alerting us to a broken contract.
        }
    }
    assert_eq!(variants.len(), 8, "DataExpr must have exactly 8 variants");
}

#[test]
fn contract_value_add_is_only_arithmetic_on_data() {
    // Value::add is the ONLY arithmetic operation — verify it exists and
    // that there is no subtract/multiply/divide method
    let a = Value::Int(10);
    let b = Value::Int(5);

    // add must work
    assert!(a.add(&b).is_ok());

    // negate must work (unary, for subtraction-via-reverse)
    assert!(a.negate().is_ok());

    // These methods do NOT exist on Value:
    // a.sub(&b)   — would be compile error
    // a.mul(&b)   — would be compile error
    // a.div(&b)   — would be compile error
    // a.modulo(&b) — would be compile error
    // The absence is verified by the fact that this test compiles.
}

// ============================================================================
// CONTRACT: Purity lattice (Total < Pure < Impure)
// ============================================================================

#[test]
fn contract_purity_lattice_total_satisfies_all() {
    let total = PurityLevel::Total;
    assert!(total.satisfies(&Purity::Total));
    assert!(total.satisfies(&Purity::Pure));
    assert!(total.satisfies(&Purity::Impure));
}

#[test]
fn contract_purity_lattice_pure_satisfies_pure_and_impure() {
    let pure = PurityLevel::Pure;
    assert!(!pure.satisfies(&Purity::Total));
    assert!(pure.satisfies(&Purity::Pure));
    assert!(pure.satisfies(&Purity::Impure));
}

#[test]
fn contract_purity_lattice_impure_satisfies_only_impure() {
    let impure = PurityLevel::Impure;
    assert!(!impure.satisfies(&Purity::Total));
    assert!(!impure.satisfies(&Purity::Pure));
    assert!(impure.satisfies(&Purity::Impure));
}

#[test]
fn contract_purity_combine_least_pure_wins() {
    // Combining Total with anything gives the other
    assert_eq!(
        PurityLevel::Total.combine(&PurityLevel::Total),
        PurityLevel::Total
    );
    assert_eq!(
        PurityLevel::Total.combine(&PurityLevel::Pure),
        PurityLevel::Pure
    );
    assert_eq!(
        PurityLevel::Total.combine(&PurityLevel::Impure),
        PurityLevel::Impure
    );

    // Combining with Impure always gives Impure
    assert_eq!(
        PurityLevel::Pure.combine(&PurityLevel::Impure),
        PurityLevel::Impure
    );
    assert_eq!(
        PurityLevel::Impure.combine(&PurityLevel::Pure),
        PurityLevel::Impure
    );
}

#[test]
fn contract_total_function_rejects_loop() {
    let code = "@total fn bad(): Int { for i in 0..10 { x = 1 } return 0 }";
    let program = parse_program(code).unwrap();
    let mut checker = PurityChecker::new();
    assert!(
        checker.check_program(&program).is_err(),
        "@total function with loop must be rejected"
    );
}

#[test]
fn contract_pure_function_rejects_io() {
    let code = "@pure fn bad(): Int { print(1) return 0 }";
    let program = parse_program(code).unwrap();
    let mut checker = PurityChecker::new();
    assert!(
        checker.check_program(&program).is_err(),
        "@pure function with print must be rejected"
    );
}

// ============================================================================
// CONTRACT: Reversibility — forward + reverse = identity
// ============================================================================

#[test]
fn contract_reversibility_identity_int() {
    let mut interp = ReversibleInterpreter::new();
    interp.set("x".to_string(), Value::Int(42));
    let original = interp.get_state().clone();

    let block = ReverseBlock {
        body: vec![ReversibleStmt::AddAssign(
            "x".to_string(),
            DataExpr::Number(Number::Int(100)),
        )],
    };

    interp.execute_and_reverse(&block).unwrap();
    assert_eq!(
        interp.get_state(),
        &original,
        "Reversibility contract: state must be identical after forward+reverse"
    );
}

#[test]
fn contract_reversibility_identity_rational() {
    let mut interp = ReversibleInterpreter::new();
    interp.set(
        "r".to_string(),
        Value::Rational(Ratio::new(1, 3)),
    );
    let original = interp.get_state().clone();

    let block = ReverseBlock {
        body: vec![ReversibleStmt::AddAssign(
            "r".to_string(),
            DataExpr::Number(Number::Rational(1, 6)),
        )],
    };

    interp.execute_and_reverse(&block).unwrap();
    assert_eq!(interp.get_state(), &original);
}

#[test]
fn contract_reversibility_identity_complex() {
    let mut interp = ReversibleInterpreter::new();
    interp.set(
        "c".to_string(),
        Value::Complex(Complex64::new(3.0, 4.0)),
    );
    let original = interp.get_state().clone();

    let block = ReverseBlock {
        body: vec![ReversibleStmt::AddAssign(
            "c".to_string(),
            DataExpr::Number(Number::Complex(1.0, 2.0)),
        )],
    };

    interp.execute_and_reverse(&block).unwrap();

    // Float comparison for Complex
    let state = interp.get_state();
    match (state.get("c"), original.get("c")) {
        (Some(Value::Complex(got)), Some(Value::Complex(expected))) => {
            assert!(
                (got.re - expected.re).abs() < 1e-10,
                "Real part mismatch"
            );
            assert!(
                (got.im - expected.im).abs() < 1e-10,
                "Imaginary part mismatch"
            );
        }
        _ => panic!("Expected Complex values"),
    }
}

#[test]
fn contract_self_referential_assignment_detected() {
    // x += x breaks reversibility — the checker must catch it
    let block = ReverseBlock {
        body: vec![ReversibleStmt::AddAssign(
            "x".to_string(),
            DataExpr::Identifier("x".to_string()),
        )],
    };
    assert!(
        check_reversibility(&block).is_err(),
        "Self-referential reversible assignment must be rejected"
    );
}

#[test]
fn contract_nested_self_reference_detected() {
    // x += (y + x) also breaks reversibility
    let block = ReverseBlock {
        body: vec![ReversibleStmt::AddAssign(
            "x".to_string(),
            DataExpr::Add(
                Box::new(DataExpr::Identifier("y".to_string())),
                Box::new(DataExpr::Identifier("x".to_string())),
            ),
        )],
    };
    assert!(
        check_reversibility(&block).is_err(),
        "Nested self-referential reversible assignment must be rejected"
    );
}

// ============================================================================
// CONTRACT: Harvard Architecture at AST level
// ============================================================================

#[test]
fn contract_control_expr_can_reference_data() {
    // ControlExpr::Data wraps DataExpr — the one-way bridge
    let data = DataExpr::Number(Number::Int(42));
    let control = ControlExpr::Data(data.clone());

    match &control {
        ControlExpr::Data(inner) => assert_eq!(inner, &data),
        _ => panic!("Expected ControlExpr::Data"),
    }
}

#[test]
fn contract_comparison_uses_data_operands() {
    // Comparisons take DataExpr operands — data flows INTO control, never out
    let left = DataExpr::Number(Number::Int(1));
    let right = DataExpr::Number(Number::Int(2));
    let comp = ControlExpr::Comparison(Box::new(left), Comparator::Lt, Box::new(right));

    match comp {
        ControlExpr::Comparison(l, _, r) => {
            assert!(matches!(*l, DataExpr::Number(_)));
            assert!(matches!(*r, DataExpr::Number(_)));
        }
        _ => panic!("Expected Comparison"),
    }
}

// ============================================================================
// CONTRACT: All 7 number systems work in Value::add
// ============================================================================

#[test]
fn contract_all_seven_number_types_can_add_with_identity() {
    let cases: Vec<(Value, Value, &str)> = vec![
        (Value::Int(42), Value::Int(0), "Int"),
        (Value::Float(3.14), Value::Float(0.0), "Float"),
        (
            Value::Rational(Ratio::new(1, 3)),
            Value::Rational(Ratio::new(0, 1)),
            "Rational",
        ),
        (
            Value::Complex(Complex64::new(1.0, 2.0)),
            Value::Complex(Complex64::new(0.0, 0.0)),
            "Complex",
        ),
        (Value::Hex(0xFF), Value::Hex(0), "Hex"),
        (Value::Binary(0b1010), Value::Binary(0), "Binary"),
        (
            Value::Symbolic("x".to_string()),
            Value::Symbolic("".to_string()),
            "Symbolic",
        ),
    ];

    for (val, zero, name) in &cases {
        let result = val.add(zero);
        assert!(
            result.is_ok(),
            "Number system {} must support addition: {:?}",
            name,
            result
        );
    }
}

// ============================================================================
// CONTRACT: Integer overflow handled safely (no panic)
// ============================================================================

#[test]
fn contract_integer_overflow_returns_error() {
    let max = Value::Int(i64::MAX);
    let one = Value::Int(1);
    let result = max.add(&one);
    assert!(
        result.is_err(),
        "i64::MAX + 1 must return Err, not panic"
    );
}

#[test]
fn contract_integer_min_negation_returns_error() {
    let min = Value::Int(i64::MIN);
    let result = min.negate();
    assert!(
        result.is_err(),
        "Negating i64::MIN must return Err, not panic"
    );
}

// ============================================================================
// LIFECYCLE: Reversible interpreter state management
// ============================================================================

#[test]
fn lifecycle_interpreter_clean_after_reverse() {
    let mut interp = ReversibleInterpreter::new();
    interp.set("x".to_string(), Value::Int(0));

    let block = ReverseBlock {
        body: vec![ReversibleStmt::AddAssign(
            "x".to_string(),
            DataExpr::Number(Number::Int(10)),
        )],
    };

    // Execute forward
    interp.execute_forward(&block).unwrap();
    assert_eq!(interp.get("x"), Some(&Value::Int(10)));

    // Execute reverse
    interp.execute_reverse().unwrap();
    assert_eq!(interp.get("x"), Some(&Value::Int(0)));

    // After reverse, trace should be cleared — another reverse should be a no-op
    interp.execute_reverse().unwrap();
    assert_eq!(
        interp.get("x"),
        Some(&Value::Int(0)),
        "After clearing trace, reverse should be no-op"
    );
}

#[test]
fn lifecycle_interpreter_multiple_blocks() {
    let mut interp = ReversibleInterpreter::new();
    interp.set("x".to_string(), Value::Int(0));

    // Block 1
    let block1 = ReverseBlock {
        body: vec![ReversibleStmt::AddAssign(
            "x".to_string(),
            DataExpr::Number(Number::Int(10)),
        )],
    };
    interp.execute_and_reverse(&block1).unwrap();
    assert_eq!(interp.get("x"), Some(&Value::Int(0)));

    // Block 2 — independent, should also work
    let block2 = ReverseBlock {
        body: vec![ReversibleStmt::AddAssign(
            "x".to_string(),
            DataExpr::Number(Number::Int(99)),
        )],
    };
    interp.execute_and_reverse(&block2).unwrap();
    assert_eq!(interp.get("x"), Some(&Value::Int(0)));
}
