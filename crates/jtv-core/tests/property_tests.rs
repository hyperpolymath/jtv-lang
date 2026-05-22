// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Property-based tests for Julia the Viper's 7 number systems.
// Verifies algebraic laws (commutativity, associativity, identity) hold
// across all number types and cross-type coercion paths.

use jtv_core::ast::*;
use jtv_core::number::Value;
use jtv_core::reversible::ReversibleInterpreter;
use jtv_core::ReverseBlock;
use num_complex::Complex64;
use num_rational::Ratio;
use proptest::prelude::*;

// ============================================================================
// Strategy generators for each of the 7 number systems
// ============================================================================

/// Generate arbitrary Int values (avoiding overflow-prone extremes)
fn arb_int() -> impl Strategy<Value = Value> {
    (-1_000_000i64..1_000_000i64).prop_map(Value::Int)
}

/// Generate arbitrary Float values (finite only — NaN/Inf break equality)
fn arb_float() -> impl Strategy<Value = Value> {
    (-1e6f64..1e6f64)
        .prop_filter("must be finite", |f| f.is_finite())
        .prop_map(Value::Float)
}

/// Generate arbitrary Rational values (non-zero denominator)
fn arb_rational() -> impl Strategy<Value = Value> {
    ((-1000i64..1000i64), (1i64..1000i64)).prop_map(|(n, d)| Value::Rational(Ratio::new(n, d)))
}

/// Generate arbitrary Complex values (finite components)
fn arb_complex() -> impl Strategy<Value = Value> {
    ((-1000.0f64..1000.0f64), (-1000.0f64..1000.0f64))
        .prop_filter("must be finite", |(r, i)| r.is_finite() && i.is_finite())
        .prop_map(|(r, i)| Value::Complex(Complex64::new(r, i)))
}

/// Generate arbitrary Hex values
fn arb_hex() -> impl Strategy<Value = Value> {
    (0i64..0xFFFF).prop_map(Value::Hex)
}

/// Generate arbitrary Binary values
fn arb_binary() -> impl Strategy<Value = Value> {
    (0i64..0xFF).prop_map(Value::Binary)
}

/// Generate arbitrary Symbolic values
fn arb_symbolic() -> impl Strategy<Value = Value> {
    "[a-z]{1,4}".prop_map(|s| Value::Symbolic(s))
}

/// Generate a Value of any number type (for cross-type tests)
fn arb_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        arb_int(),
        arb_float(),
        arb_rational(),
        arb_complex(),
        arb_hex(),
        arb_binary(),
        arb_symbolic(),
    ]
}

// ============================================================================
// Commutativity: a + b == b + a
// ============================================================================

proptest! {
    #[test]
    fn prop_int_addition_commutative(a in arb_int(), b in arb_int()) {
        let ab = a.add(&b);
        let ba = b.add(&a);
        match (ab, ba) {
            (Ok(ab), Ok(ba)) => prop_assert_eq!(ab, ba),
            _ => {} // Both overflow is acceptable
        }
    }

    #[test]
    fn prop_float_addition_commutative(a in arb_float(), b in arb_float()) {
        let ab = a.add(&b).unwrap();
        let ba = b.add(&a).unwrap();
        prop_assert_eq!(ab, ba);
    }

    #[test]
    fn prop_rational_addition_commutative(a in arb_rational(), b in arb_rational()) {
        let ab = a.add(&b).unwrap();
        let ba = b.add(&a).unwrap();
        prop_assert_eq!(ab, ba);
    }

    #[test]
    fn prop_complex_addition_commutative(a in arb_complex(), b in arb_complex()) {
        let ab = a.add(&b).unwrap();
        let ba = b.add(&a).unwrap();
        prop_assert_eq!(ab, ba);
    }

    #[test]
    fn prop_hex_addition_commutative(a in arb_hex(), b in arb_hex()) {
        let ab = a.add(&b);
        let ba = b.add(&a);
        match (ab, ba) {
            (Ok(ab), Ok(ba)) => prop_assert_eq!(ab, ba),
            _ => {}
        }
    }

    #[test]
    fn prop_binary_addition_commutative(a in arb_binary(), b in arb_binary()) {
        let ab = a.add(&b);
        let ba = b.add(&a);
        match (ab, ba) {
            (Ok(ab), Ok(ba)) => prop_assert_eq!(ab, ba),
            _ => {}
        }
    }

    #[test]
    fn prop_symbolic_addition_commutative_structure(
        a in arb_symbolic(),
        b in arb_symbolic()
    ) {
        // Symbolic addition produces "a + b" string — commutativity is structural,
        // not semantic (we don't simplify symbolic expressions)
        let ab = a.add(&b).unwrap();
        let ba = b.add(&a).unwrap();
        // Both should be Symbolic variants (structural property)
        prop_assert!(matches!(ab, Value::Symbolic(_)));
        prop_assert!(matches!(ba, Value::Symbolic(_)));
    }
}

// ============================================================================
// Associativity: (a + b) + c == a + (b + c)
// ============================================================================

proptest! {
    #[test]
    fn prop_int_addition_associative(
        a in (-10_000i64..10_000i64).prop_map(Value::Int),
        b in (-10_000i64..10_000i64).prop_map(Value::Int),
        c in (-10_000i64..10_000i64).prop_map(Value::Int),
    ) {
        let ab_c = a.add(&b).and_then(|ab| ab.add(&c));
        let a_bc = b.add(&c).and_then(|bc| a.add(&bc));
        match (ab_c, a_bc) {
            (Ok(left), Ok(right)) => prop_assert_eq!(left, right),
            _ => {} // Overflow on both sides is acceptable
        }
    }

    #[test]
    fn prop_rational_addition_associative(
        a in arb_rational(),
        b in arb_rational(),
        c in arb_rational(),
    ) {
        let ab_c = a.add(&b).and_then(|ab| ab.add(&c)).unwrap();
        let a_bc = b.add(&c).and_then(|bc| a.add(&bc)).unwrap();
        prop_assert_eq!(ab_c, a_bc);
    }

    #[test]
    fn prop_complex_addition_associative(
        a in arb_complex(),
        b in arb_complex(),
        c in arb_complex(),
    ) {
        let ab_c = a.add(&b).and_then(|ab| ab.add(&c)).unwrap();
        let a_bc = b.add(&c).and_then(|bc| a.add(&bc)).unwrap();
        // Floating-point: check approximate equality
        match (&ab_c, &a_bc) {
            (Value::Complex(l), Value::Complex(r)) => {
                prop_assert!((l.re - r.re).abs() < 1e-6, "real parts differ: {} vs {}", l.re, r.re);
                prop_assert!((l.im - r.im).abs() < 1e-6, "imaginary parts differ: {} vs {}", l.im, r.im);
            }
            _ => prop_assert!(false, "Expected Complex values"),
        }
    }
}

// ============================================================================
// Identity: a + 0 == a
// ============================================================================

proptest! {
    #[test]
    fn prop_int_addition_identity(a in arb_int()) {
        let zero = Value::Int(0);
        let result = a.add(&zero).unwrap();
        prop_assert_eq!(result, a);
    }

    #[test]
    fn prop_float_addition_identity(a in arb_float()) {
        let zero = Value::Float(0.0);
        let result = a.add(&zero).unwrap();
        prop_assert_eq!(result, a);
    }

    #[test]
    fn prop_rational_addition_identity(a in arb_rational()) {
        let zero = Value::Rational(Ratio::new(0, 1));
        let result = a.add(&zero).unwrap();
        prop_assert_eq!(result, a);
    }

    #[test]
    fn prop_complex_addition_identity(a in arb_complex()) {
        let zero = Value::Complex(Complex64::new(0.0, 0.0));
        let result = a.add(&zero).unwrap();
        prop_assert_eq!(result, a);
    }

    #[test]
    fn prop_hex_addition_identity(a in arb_hex()) {
        let zero = Value::Hex(0);
        let result = a.add(&zero).unwrap();
        prop_assert_eq!(result, a);
    }

    #[test]
    fn prop_binary_addition_identity(a in arb_binary()) {
        let zero = Value::Binary(0);
        let result = a.add(&zero).unwrap();
        prop_assert_eq!(result, a);
    }
}

// ============================================================================
// Inverse: a + (-a) == 0
// ============================================================================

proptest! {
    #[test]
    fn prop_int_additive_inverse(a in (-500_000i64..500_000i64).prop_map(Value::Int)) {
        let neg = a.negate().unwrap();
        let result = a.add(&neg).unwrap();
        prop_assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn prop_float_additive_inverse(a in arb_float()) {
        let neg = a.negate().unwrap();
        let result = a.add(&neg).unwrap();
        match result {
            Value::Float(f) => prop_assert!(f.abs() < 1e-10, "Expected ~0.0, got {}", f),
            _ => prop_assert!(false, "Expected Float"),
        }
    }

    #[test]
    fn prop_rational_additive_inverse(a in arb_rational()) {
        let neg = a.negate().unwrap();
        let result = a.add(&neg).unwrap();
        prop_assert_eq!(result, Value::Rational(Ratio::new(0, 1)));
    }

    #[test]
    fn prop_complex_additive_inverse(a in arb_complex()) {
        let neg = a.negate().unwrap();
        let result = a.add(&neg).unwrap();
        match result {
            Value::Complex(c) => {
                prop_assert!(c.re.abs() < 1e-10, "Real part not ~0: {}", c.re);
                prop_assert!(c.im.abs() < 1e-10, "Imag part not ~0: {}", c.im);
            }
            _ => prop_assert!(false, "Expected Complex"),
        }
    }
}

// ============================================================================
// Cross-type coercion properties
// ============================================================================

proptest! {
    #[test]
    fn prop_int_float_coercion_commutative(
        i in (-1000i64..1000i64),
        f in (-1000.0f64..1000.0f64).prop_filter("finite", |f| f.is_finite()),
    ) {
        let int_val = Value::Int(i);
        let float_val = Value::Float(f);
        let if_result = int_val.add(&float_val).unwrap();
        let fi_result = float_val.add(&int_val).unwrap();
        // Both should produce Float
        prop_assert!(matches!(if_result, Value::Float(_)));
        prop_assert!(matches!(fi_result, Value::Float(_)));
        prop_assert_eq!(if_result, fi_result);
    }

    #[test]
    fn prop_int_rational_coercion_commutative(
        i in (-1000i64..1000i64),
        n in (-100i64..100i64),
        d in (1i64..100i64),
    ) {
        let int_val = Value::Int(i);
        let rat_val = Value::Rational(Ratio::new(n, d));
        let ir_result = int_val.add(&rat_val).unwrap();
        let ri_result = rat_val.add(&int_val).unwrap();
        // Both should produce Rational
        prop_assert!(matches!(ir_result, Value::Rational(_)));
        prop_assert!(matches!(ri_result, Value::Rational(_)));
        prop_assert_eq!(ir_result, ri_result);
    }

    #[test]
    fn prop_float_complex_coercion_commutative(
        f in (-1000.0f64..1000.0f64).prop_filter("finite", |f| f.is_finite()),
        re in (-1000.0f64..1000.0f64).prop_filter("finite", |f| f.is_finite()),
        im in (-1000.0f64..1000.0f64).prop_filter("finite", |f| f.is_finite()),
    ) {
        let float_val = Value::Float(f);
        let complex_val = Value::Complex(Complex64::new(re, im));
        let fc_result = float_val.add(&complex_val).unwrap();
        let cf_result = complex_val.add(&float_val).unwrap();
        // Both should produce Complex
        prop_assert!(matches!(fc_result, Value::Complex(_)));
        prop_assert!(matches!(cf_result, Value::Complex(_)));
        prop_assert_eq!(fc_result, cf_result);
    }
}

// ============================================================================
// Negation properties
// ============================================================================

proptest! {
    #[test]
    fn prop_double_negation_is_identity_int(a in (-500_000i64..500_000i64).prop_map(Value::Int)) {
        let neg_neg = a.negate().and_then(|n| n.negate()).unwrap();
        prop_assert_eq!(neg_neg, a);
    }

    #[test]
    fn prop_double_negation_is_identity_float(a in arb_float()) {
        let neg_neg = a.negate().and_then(|n| n.negate()).unwrap();
        prop_assert_eq!(neg_neg, a);
    }

    #[test]
    fn prop_double_negation_is_identity_rational(a in arb_rational()) {
        let neg_neg = a.negate().and_then(|n| n.negate()).unwrap();
        prop_assert_eq!(neg_neg, a);
    }

    #[test]
    fn prop_double_negation_is_identity_complex(a in arb_complex()) {
        let neg_neg = a.negate().and_then(|n| n.negate()).unwrap();
        prop_assert_eq!(neg_neg, a);
    }
}

// ============================================================================
// Overflow safety: checked arithmetic never panics
// ============================================================================

proptest! {
    #[test]
    fn prop_int_addition_never_panics(a in any::<i64>(), b in any::<i64>()) {
        let va = Value::Int(a);
        let vb = Value::Int(b);
        // Must not panic — either Ok or Err(IntegerOverflow)
        let _ = va.add(&vb);
    }

    #[test]
    fn prop_int_negation_never_panics(a in any::<i64>()) {
        let va = Value::Int(a);
        // i64::MIN negation should return Err, not panic
        let _ = va.negate();
    }

    #[test]
    fn prop_hex_addition_never_panics(a in any::<i64>(), b in any::<i64>()) {
        let va = Value::Hex(a);
        let vb = Value::Hex(b);
        let _ = va.add(&vb);
    }

    #[test]
    fn prop_binary_addition_never_panics(a in any::<i64>(), b in any::<i64>()) {
        let va = Value::Binary(a);
        let vb = Value::Binary(b);
        let _ = va.add(&vb);
    }
}

// ============================================================================
// Reversibility: forward + reverse = identity (THE key JTV property)
// ============================================================================

proptest! {
    #[test]
    fn prop_reversibility_single_add(
        initial in (-100_000i64..100_000i64),
        delta in (-10_000i64..10_000i64),
    ) {
        let mut interp = ReversibleInterpreter::new();
        interp.set("x".to_string(), Value::Int(initial));

        let block = ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::Number(Number::Int(delta)),
            )],
        };

        interp.execute_and_reverse(&block).unwrap();
        prop_assert_eq!(interp.get("x"), Some(&Value::Int(initial)));
    }

    #[test]
    fn prop_reversibility_multi_variable(
        x0 in (-10_000i64..10_000i64),
        y0 in (-10_000i64..10_000i64),
        dx in (-1_000i64..1_000i64),
        dy in (-1_000i64..1_000i64),
    ) {
        let mut interp = ReversibleInterpreter::new();
        interp.set("x".to_string(), Value::Int(x0));
        interp.set("y".to_string(), Value::Int(y0));

        let block = ReverseBlock {
            body: vec![
                ReversibleStmt::AddAssign("x".to_string(), DataExpr::Number(Number::Int(dx))),
                ReversibleStmt::AddAssign("y".to_string(), DataExpr::Number(Number::Int(dy))),
            ],
        };

        interp.execute_and_reverse(&block).unwrap();
        prop_assert_eq!(interp.get("x"), Some(&Value::Int(x0)));
        prop_assert_eq!(interp.get("y"), Some(&Value::Int(y0)));
    }

    #[test]
    fn prop_reversibility_cross_variable(
        x0 in (-1_000i64..1_000i64),
        y0 in (-1_000i64..1_000i64),
    ) {
        // x += y is reversible because y is independent of x
        let mut interp = ReversibleInterpreter::new();
        interp.set("x".to_string(), Value::Int(x0));
        interp.set("y".to_string(), Value::Int(y0));

        let block = ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::Identifier("y".to_string()),
            )],
        };

        interp.execute_and_reverse(&block).unwrap();
        prop_assert_eq!(interp.get("x"), Some(&Value::Int(x0)));
        prop_assert_eq!(interp.get("y"), Some(&Value::Int(y0)));
    }

    #[test]
    fn prop_reversibility_chain_preserves_state(
        a0 in (-500i64..500i64),
        b0 in (-500i64..500i64),
        d1 in (-100i64..100i64),
        d2 in (-100i64..100i64),
        d3 in (-100i64..100i64),
    ) {
        let mut interp = ReversibleInterpreter::new();
        interp.set("a".to_string(), Value::Int(a0));
        interp.set("b".to_string(), Value::Int(b0));

        let block = ReverseBlock {
            body: vec![
                ReversibleStmt::AddAssign("a".to_string(), DataExpr::Number(Number::Int(d1))),
                ReversibleStmt::SubAssign("b".to_string(), DataExpr::Number(Number::Int(d2))),
                ReversibleStmt::AddAssign("a".to_string(), DataExpr::Number(Number::Int(d3))),
            ],
        };

        let original_a = interp.get("a").cloned();
        let original_b = interp.get("b").cloned();

        interp.execute_and_reverse(&block).unwrap();

        prop_assert_eq!(interp.get("a").cloned(), original_a);
        prop_assert_eq!(interp.get("b").cloned(), original_b);
    }
}

// ============================================================================
// Parser round-trip: parse -> format -> parse == parse
// ============================================================================

proptest! {
    #[test]
    fn prop_number_from_ast_roundtrip_int(n in any::<i64>()) {
        let num = Number::Int(n);
        let val = Value::from_number(&num).unwrap();
        prop_assert_eq!(val, Value::Int(n));
    }

    #[test]
    fn prop_number_from_ast_roundtrip_float(n in any::<f64>().prop_filter("finite", |f| f.is_finite())) {
        let num = Number::Float(n);
        let val = Value::from_number(&num).unwrap();
        prop_assert_eq!(val, Value::Float(n));
    }

    #[test]
    fn prop_number_from_ast_roundtrip_rational(
        n in (-10_000i64..10_000i64),
        d in (1i64..10_000i64),
    ) {
        let num = Number::Rational(n, d);
        let val = Value::from_number(&num).unwrap();
        // Ratio normalises, so check equivalence via cross-multiplication
        match val {
            Value::Rational(r) => {
                prop_assert_eq!(*r.numer() * d, n * *r.denom());
            }
            _ => prop_assert!(false, "Expected Rational"),
        }
    }
}

// ============================================================================
// Type error consistency: incompatible types always fail, never panic
// ============================================================================

proptest! {
    #[test]
    fn prop_bool_cannot_add(b in any::<bool>()) {
        let val = Value::Bool(b);
        let other = Value::Int(1);
        let result = val.add(&other);
        prop_assert!(result.is_err());
    }

    #[test]
    fn prop_unit_cannot_add(_dummy in 0..1u8) {
        let val = Value::Unit;
        let other = Value::Int(1);
        let result = val.add(&other);
        prop_assert!(result.is_err());
    }

    #[test]
    fn prop_bool_cannot_negate(b in any::<bool>()) {
        let val = Value::Bool(b);
        let result = val.negate();
        prop_assert!(result.is_err());
    }
}
