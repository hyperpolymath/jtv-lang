// SPDX-License-Identifier: MPL-2.0
//
// Reversal grading by the carrier a step produces
// (`theory/agda/src/Jtv/Promotion.agda`, `echo::step_echo`).
//
// A step whose operand promotes the target to another carrier cannot be undone
// by arithmetic inversion: `Int += Float` rounds, and `Int += Rational` leaves a
// Rational where the Int stood. Such a step is never `Safe`, so a tokenless
// `reverse { }` rejects it; a token-bound block admits it and restores the old
// value from a snapshot.

use jtv_core::ast::BasicType;
use jtv_core::echo::{carrier_echo, promote, step_echo, Echo};
use jtv_core::{number::Value, parser::parse_program, Interpreter, JtvError, TypeChecker};

const CARRIERS: [BasicType; 7] = [
    BasicType::Int,
    BasicType::Rational,
    BasicType::Float,
    BasicType::Complex,
    BasicType::Hex,
    BasicType::Binary,
    BasicType::Symbolic,
];

/// One runtime value of each numeric carrier.
fn sample(c: &BasicType) -> Value {
    match c {
        BasicType::Int => Value::Int(3),
        BasicType::Rational => Value::Rational(num_rational::Ratio::new(1, 2)),
        BasicType::Float => Value::Float(0.5),
        BasicType::Complex => Value::Complex(num_complex::Complex64::new(0.5, 0.25)),
        BasicType::Hex => Value::Hex(3),
        BasicType::Binary => Value::Binary(3),
        BasicType::Symbolic => Value::Symbolic("a".into()),
        other => panic!("not a numeric carrier: {other:?}"),
    }
}

fn carrier_of_value(v: &Value) -> BasicType {
    match v {
        Value::Int(_) => BasicType::Int,
        Value::Rational(_) => BasicType::Rational,
        Value::Float(_) => BasicType::Float,
        Value::Complex(_) => BasicType::Complex,
        Value::Hex(_) => BasicType::Hex,
        Value::Binary(_) => BasicType::Binary,
        Value::Symbolic(_) => BasicType::Symbolic,
        other => panic!("not a numeric value: {other:?}"),
    }
}

fn check(src: &str) -> Result<(), JtvError> {
    let prog = parse_program(src).expect("should parse");
    TypeChecker::new().check_program(&prog)
}

fn run(src: &str) -> Interpreter {
    let prog = parse_program(src).expect("should parse");
    let mut interp = Interpreter::new();
    interp.run(&prog).expect("should run");
    interp
}

fn get_var(interp: &Interpreter, name: &str) -> Option<Value> {
    interp
        .get_variables()
        .into_iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v)
}

// The promotion table the grade is proved over is the one the runtime uses:
// for all 49 carrier pairs, `promote` is `Some(c)` exactly when `Value::add`
// succeeds, and `c` is the carrier of the sum.
#[test]
fn promote_agrees_with_value_add_on_every_carrier_pair() {
    for a in &CARRIERS {
        for b in &CARRIERS {
            let runtime = sample(a).add(&sample(b)).ok().map(|v| carrier_of_value(&v));
            assert_eq!(promote(a, b), runtime, "promote({a:?}, {b:?})");
        }
    }
}

// `promote-absorb`: adding the same operand again does not promote further.
#[test]
fn promote_absorbs_a_repeated_operand() {
    for a in &CARRIERS {
        for b in &CARRIERS {
            if let Some(c) = promote(a, b) {
                assert_eq!(promote(&c, b), Some(c.clone()), "promote({a:?}, {b:?})");
            }
        }
    }
}

// `result-bounds-grade`, `target-bounds-grade` and `safe-grade-sound`, checked
// over every shape, target and result.
#[test]
fn step_echo_satisfies_the_proved_bounds() {
    for shape in [Echo::Safe, Echo::Neutral, Echo::Breaking] {
        for target in &CARRIERS {
            for result in &CARRIERS {
                let grade = step_echo(shape, target, Some(result));
                assert_eq!(grade.join(carrier_echo(result)), grade);
                assert_eq!(grade.join(carrier_echo(target)), grade);
                if grade == Echo::Safe {
                    assert_eq!(shape, Echo::Safe);
                    assert_eq!(carrier_echo(target), Echo::Safe);
                    assert_eq!(target, result);
                }
                if target != result {
                    assert_ne!(grade, Echo::Safe);
                }
            }
            // An unknown result fails closed.
            assert_ne!(step_echo(shape, target, None), Echo::Safe);
        }
    }
}

// The reported defect: `reverse { x += 0.5 }` over an Int `x` was accepted, and
// running it turned 9007199254740993 into 9007199254740992.
#[test]
fn tokenless_reverse_rejects_int_promoted_to_float() {
    for op in ["+=", "-="] {
        let src = format!("x = 9007199254740993\nreverse {{\n  x {op} 0.5\n}}\n");
        assert!(
            matches!(check(&src), Err(JtvError::EchoViolation(_))),
            "{op}"
        );
    }
}

// `Int += Rational` is exact arithmetic but still changes the carrier.
#[test]
fn tokenless_reverse_rejects_int_promoted_to_rational() {
    for op in ["+=", "-="] {
        let src = format!("x = 1\nreverse {{\n  x {op} 1/2\n}}\n");
        assert!(
            matches!(check(&src), Err(JtvError::EchoViolation(_))),
            "{op}"
        );
    }
}

// The promotion can hide inside the operand expression or behind a variable.
#[test]
fn tokenless_reverse_rejects_promotion_through_expressions() {
    assert!(matches!(
        check("x = 1\ny = 0.5\nreverse {\n  x += y\n}\n"),
        Err(JtvError::EchoViolation(_))
    ));
    assert!(matches!(
        check("x = 1\nreverse {\n  x += 1 + 0.5\n}\n"),
        Err(JtvError::EchoViolation(_))
    ));
    assert!(matches!(
        check("x = 1\nreverse {\n  x += -0.5\n}\n"),
        Err(JtvError::EchoViolation(_))
    ));
}

// Steps that keep the carrier stay admissible, so the rule adds no false
// rejections: Int += Int, and Rational += Int (the Int embeds; no promotion).
#[test]
fn tokenless_reverse_admits_steps_that_keep_the_carrier() {
    check("x = 1\nreverse {\n  x += 2\n}\n").expect("Int += Int");
    check("x = 1\ny = 2\nreverse {\n  x += y + 3\n}\n").expect("Int += Int expr");
    check("x = 1/2\nreverse {\n  x += 2\n}\n").expect("Rational += Int");
}

// A call operand is graded by the carrier the callee returns.
// A call operand has an unknown carrier: the checker does not compare `return`
// values with a function's declared return type, so the declaration cannot be
// trusted here. The step is `Neutral`, whatever the function is declared to
// return, and the tokenless gate rejects it.
#[test]
fn tokenless_reverse_rejects_a_call_operand() {
    for ret in ["Int", "Float"] {
        let src =
            format!("@total fn f(): {ret} {{\n  return 2\n}}\nx = 1\nreverse {{\n  x += f()\n}}\n");
        assert!(
            matches!(check(&src), Err(JtvError::EchoViolation(_))),
            "call declared to return {ret}"
        );
    }
    // The case the declaration would get wrong: declared Int, returns a Float.
    assert!(matches!(
        check("@total fn half(): Int {\n  return 0.5\n}\nx = 1\nreverse {\n  x += half()\n}\n"),
        Err(JtvError::EchoViolation(_))
    ));
}

// The reverse gate and the function-effect path grade the same block the same
// way, so `@echo(Safe)` cannot disagree with the gate about a call operand.
#[test]
fn effect_path_agrees_with_the_gate_on_a_call_operand() {
    let src = "@total fn two(): Int {\n  return 2\n}\nfn bump(x: Int): Int {\n  reverse {\n    x += two()\n  }\n  return x\n}\n";
    let program = parse_program(src).expect("parses");
    let effects = jtv_core::effect::resolved_effects(&program);
    assert_eq!(effects["bump"].echo, Echo::Neutral);
    assert!(matches!(check(src), Err(JtvError::EchoViolation(_))));
}

// A reversible step leaves its target in the carrier of the sum, and later
// blocks are graded against that carrier. After a token-bound `x += 0.5` an Int
// `x` holds a Float, so a tokenless `reverse { x += 1 }` is a Float step.
#[test]
fn a_later_block_is_graded_by_the_carrier_the_step_left() {
    assert!(matches!(
        check("x = 1\nreversible {\n  x += 0.5\n} -> tok\nreverse {\n  x += 1\n}\n"),
        Err(JtvError::EchoViolation(_))
    ));
    // Int promoted to Rational: a further Rational step keeps the carrier, so
    // the tokenless block is Safe and is admitted.
    check("x = 1\nreversible {\n  x += 1/2\n} -> tok\nreverse {\n  x += 1/2\n}\n")
        .expect("Rational += Rational after the promotion");
}

// `reverse tok` restores the old value and carrier, so the checker restores
// the type with it. Leaving the promoted type in place admitted an unsafe
// step: `x` is an Int again, and `reverse { x += 1/2 }` promotes it.
#[test]
fn reverse_tok_restores_the_carrier_the_checker_grades_by() {
    assert!(matches!(
        check("x = 1\nreversible {\n  x += 1/2\n} -> tok\nreverse tok\nreverse {\n  x += 1/2\n}\n"),
        Err(JtvError::EchoViolation(_))
    ));
    // And the Int steps the variable supports again are admitted.
    let src = "x = 1\nreversible {\n  x += 0.5\n} -> tok\nreverse tok\nreverse {\n  x += 1\n}\n";
    check(src).expect("Int += Int after the token is reversed");
    assert!(matches!(get_var(&run(src), "x"), Some(Value::Int(_))));
}

// If a target is given another type between its block and `reverse tok`, the
// carrier the reversal leaves depends on which inverse the runtime recorded,
// which the checker cannot see. Restoring the old type let a Float be graded
// as an Int; leaving the new one let an Int be graded as a Rational. The
// reversal is rejected.
#[test]
fn reverse_tok_is_rejected_after_the_target_changes_type() {
    for src in [
        "x = 1\nreversible {\n  x += 1\n} -> tok\nx = 9007199254740996.0\nreverse tok\nreverse {\n  x += 1\n}\n",
        "x = 1\nreversible {\n  x += 0.5\n} -> tok\nx = 2/3\nreverse tok\nreverse {\n  x += 1/2\n}\n",
    ] {
        assert!(
            matches!(check(src), Err(JtvError::TypeError(_))),
            "{src}"
        );
    }
    // A block that updates one target twice is reversible: the target is
    // recorded once, with the type before the block and the type after it.
    let twice = "x = 1\nreversible {\n  x += 1\n  x += 0.5\n} -> tok\nreverse tok\nreverse {\n  x += 1\n}\n";
    check(twice).expect("two steps on one target, then reverse tok");
    assert_eq!(get_var(&run(twice), "x"), Some(Value::Int(0)));
    // Reassigning with the type the block left is still reversible.
    check("x = 1\nreversible {\n  x += 1\n} -> tok\nx = 5\nreverse tok\nreverse {\n  x += 1\n}\n")
        .expect("same type as the block left");
}

// With a token the promoting step is admitted (`Neutral`), and `reverse tok`
// restores the original value and carrier from the snapshot.
#[test]
fn token_bound_promotion_is_admitted_and_restored_exactly() {
    for (setup, operand, expected) in [
        ("x = 1", "1/2", Value::Int(1)),
        ("x = 9007199254740993", "0.5", Value::Int(9007199254740993)),
        (
            "x = 1/2",
            "2",
            Value::Rational(num_rational::Ratio::new(1, 2)),
        ),
    ] {
        for op in ["+=", "-="] {
            let src = format!("{setup}\nreversible {{ x {op} {operand} }} -> tok\nreverse tok\n");
            check(&src).expect("token-bound block is admitted");
            assert_eq!(
                get_var(&run(&src), "x"),
                Some(expected.clone()),
                "{setup}; x {op} {operand}"
            );
        }
    }
}
