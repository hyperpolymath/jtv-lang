// SPDX-License-Identifier: MPL-2.0
use jtv_core::{
    echo::Echo, effect, error::JtvError, parser::parse_program, typechecker::TypeChecker,
};

#[test]
fn declared_call_carriers_agree_with_effect_annotations() {
    for (ty, value, grade) in [("Int", "1", Echo::Safe), ("Float", "1.5", Echo::Neutral)] {
        let annotation = if grade == Echo::Safe {
            "Safe"
        } else {
            "Neutral"
        };
        let src = format!(
            "@total fn operand(): {ty} {{ return {value} }}
             @echo({annotation}) fn caller(x: Int) {{
                 if 1 == 1 {{ reversible {{ x += operand() }} -> tok }}
             }}"
        );
        let program = parse_program(&src).unwrap();
        assert_eq!(effect::resolved_effects(&program)["caller"].echo, grade);
        TypeChecker::new().check_program(&program).unwrap();
    }
}

#[test]
fn returned_values_must_match_declared_carrier() {
    for src in [
        "@total fn lie(): Int { return 1.5 }",
        "@total fn lie(): Int { if 1 == 1 { return 1.5 } return 1 }",
        "fn lie(): Int { return }",
        "fn lie() { return 1 }",
    ] {
        let program = parse_program(src).unwrap();
        assert!(TypeChecker::new().check_program(&program).is_err(), "{src}");
    }
}

#[test]
fn forward_updates_change_carrier_for_later_blocks() {
    for op in ["+=", "-="] {
        let src = format!("x = 1 reversible {{ x {op} 0.5 }} -> tok reverse {{ x += 1 }}");
        let program = parse_program(&src).unwrap();
        assert!(matches!(
            TypeChecker::new().check_program(&program),
            Err(JtvError::EchoViolation(_))
        ));
        let src =
            format!("x = 1 reversible {{ x {op} 0.5 }} -> tok reversible {{ x += 1 }} -> next");
        let program = parse_program(&src).unwrap();
        TypeChecker::new().check_program(&program).unwrap();
    }
}

#[test]
fn data_calls_looping_pure_is_rejected() {
    let program =
        parse_program("@pure fn looping(): Int { while 1 == 1 {} return 1 } x = looping()")
            .unwrap();
    assert!(matches!(
        TypeChecker::new().check_program(&program),
        Err(JtvError::PurityViolation(_))
    ));
}
