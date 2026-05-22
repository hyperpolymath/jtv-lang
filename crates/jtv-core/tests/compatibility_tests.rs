// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Version compatibility tests for Julia the Viper.
// Verifies that serialized AST snapshots from earlier versions can still
// be deserialized by the current code. This prevents silent data corruption
// when AST types change.
//
// Taxonomy category: Compatibility (CMP), Versability (VER)

use jtv_core::ast::*;
use jtv_core::parser::parse_program;

// ============================================================================
// AST serialization round-trip: parse -> serialize -> deserialize -> compare
// ============================================================================

#[test]
fn compat_ast_json_roundtrip_simple_assignment() {
    let code = "x = 5 + 3";
    let program = parse_program(code).unwrap();

    let json = serde_json::to_string(&program).unwrap();
    let deserialized: Program = serde_json::from_str(&json).unwrap();

    assert_eq!(program, deserialized);
}

#[test]
fn compat_ast_json_roundtrip_all_number_systems() {
    let programs = vec![
        "a = 42",
        "b = 3.14",
        "c = 1/3",
        "d = 2+3i",
        "e = 0xFF",
        "f = 0b1010",
        "g = #pi",
    ];

    for code in programs {
        let program = parse_program(code).unwrap();
        let json = serde_json::to_string(&program).unwrap();
        let deserialized: Program = serde_json::from_str(&json).unwrap();
        assert_eq!(program, deserialized, "Round-trip failed for: {}", code);
    }
}

#[test]
fn compat_ast_json_roundtrip_control_flow() {
    let code = r#"
        if x > 10 {
            y = x + 1
        } else {
            y = 0
        }
    "#;
    let program = parse_program(code).unwrap();
    let json = serde_json::to_string(&program).unwrap();
    let deserialized: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, deserialized);
}

#[test]
fn compat_ast_json_roundtrip_function() {
    let code = "@pure fn add(a: Int, b: Int): Int { return a + b }";
    let program = parse_program(code).unwrap();
    let json = serde_json::to_string(&program).unwrap();
    let deserialized: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, deserialized);
}

#[test]
fn compat_ast_json_roundtrip_reverse_block() {
    let code = "reverse { x += 5 }";
    let program = parse_program(code).unwrap();
    let json = serde_json::to_string(&program).unwrap();
    let deserialized: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, deserialized);
}

#[test]
fn compat_ast_json_roundtrip_string_literal() {
    let code = r#"x = "hello world""#;
    let program = parse_program(code).unwrap();
    let json = serde_json::to_string(&program).unwrap();
    let deserialized: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, deserialized);
}

#[test]
fn compat_ast_json_roundtrip_module() {
    let code = "module Geometry { @pure fn area(a: Int, b: Int): Int { return a + b } }";
    let program = parse_program(code).unwrap();
    let json = serde_json::to_string(&program).unwrap();
    let deserialized: Program = serde_json::from_str(&json).unwrap();
    assert_eq!(program, deserialized);
}

// ============================================================================
// Snapshot: v0.1.0 AST JSON format (frozen reference)
// If this test breaks, it means the AST serialization format changed and
// existing persisted data would be incompatible. Intentional changes should
// update the snapshot and bump the version.
// ============================================================================

#[test]
fn compat_v0_1_0_snapshot_simple_addition() {
    // This is the FROZEN v0.1.0 JSON representation of "x = 5 + 3"
    // DO NOT update this without bumping the version and documenting the change.
    let v0_1_0_json = r#"{"statements":[{"Control":{"Assignment":{"target":"x","value":{"Data":{"Add":[{"Number":{"Int":5}},{"Number":{"Int":3}}]}}}}}]}"#;

    // Current code must be able to deserialize the v0.1.0 format
    let deserialized: Program = serde_json::from_str(v0_1_0_json)
        .expect("COMPATIBILITY BREAK: Cannot deserialize v0.1.0 AST format");

    // Verify the content is correct
    assert_eq!(deserialized.statements.len(), 1);
    match &deserialized.statements[0] {
        TopLevel::Control(ControlStmt::Assignment(assign)) => {
            assert_eq!(assign.target, "x");
        }
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn compat_v0_1_0_snapshot_reverse_block() {
    // Frozen v0.1.0 JSON for "reverse { y += 10 }"
    let v0_1_0_json = r#"{"statements":[{"Control":{"ReverseBlock":{"body":[{"AddAssign":["y",{"Number":{"Int":10}}]}]}}}]}"#;

    let deserialized: Program = serde_json::from_str(v0_1_0_json)
        .expect("COMPATIBILITY BREAK: Cannot deserialize v0.1.0 reverse block format");

    assert_eq!(deserialized.statements.len(), 1);
    match &deserialized.statements[0] {
        TopLevel::Control(ControlStmt::ReverseBlock(block)) => {
            assert_eq!(block.body.len(), 1);
        }
        _ => panic!("Expected reverse block"),
    }
}

#[test]
fn compat_v0_1_0_snapshot_pure_function() {
    // Frozen v0.1.0 JSON for "@pure fn id(x: Int): Int { return x }"
    let v0_1_0_json = r#"{"statements":[{"Function":{"name":"id","params":[{"name":"x","type_annotation":{"Basic":"Int"}}],"return_type":{"Basic":"Int"},"purity":"Pure","body":[{"Return":{"Identifier":"x"}}]}}]}"#;

    let deserialized: Program = serde_json::from_str(v0_1_0_json)
        .expect("COMPATIBILITY BREAK: Cannot deserialize v0.1.0 function format");

    assert_eq!(deserialized.statements.len(), 1);
    match &deserialized.statements[0] {
        TopLevel::Function(func) => {
            assert_eq!(func.name, "id");
            assert_eq!(func.purity, Purity::Pure);
        }
        _ => panic!("Expected function"),
    }
}

// ============================================================================
// Structural: verify AST types implement required traits
// ============================================================================

#[test]
fn compat_program_is_serializable() {
    let program = Program::new();
    let json = serde_json::to_string(&program).unwrap();
    assert_eq!(json, r#"{"statements":[]}"#);
}

#[test]
fn compat_program_is_cloneable() {
    let code = "x = 1 + 2";
    let program = parse_program(code).unwrap();
    let cloned = program.clone();
    assert_eq!(program, cloned);
}
