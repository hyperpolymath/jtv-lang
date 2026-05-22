// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Integration tests for the JtV/PataCL coproc pipeline (Phase 2).
//!
//! TC1 — grammar: parse extern coproc blocks
//! TC2 — resolution: live gate keeps block; dead gate drops block
//! TC3 — namespace: surviving decls registered
//! TC4 — call-site: ExternCoprocNotYetLowered fires correctly
//! TC5 — no-pata: blocks survive when pata source absent

use jtv_core::{
    ast::{CoprocItem, TopLevel},
    coproc::{resolve_coproc_blocks, CoprocEnv},
    error::JtvError,
    interpreter::Interpreter,
    parser::parse_program,
};

// ── TC1: Grammar ────────────────────────────────────────────────────────────

#[test]
fn tc1_parse_extern_coproc_block() {
    let src = r#"
extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
}
"#;
    let prog = parse_program(src).expect("should parse");
    assert_eq!(prog.statements.len(), 1);
    match &prog.statements[0] {
        TopLevel::ExternCoproc(block) => {
            assert_eq!(block.gate_name, "riscv_vec");
            assert_eq!(block.items.len(), 1);
            assert!(block.resolved.is_none());
        }
        other => panic!("expected ExternCoproc, got {other:?}"),
    }
}

#[test]
fn tc1_parse_intrinsic_item() {
    let src = r#"
extern coproc zbb {
    @pure intrinsic clz(a: Int): Int ;
    @pure intrinsic cpop(a: Int): Int ;
}
"#;
    let prog = parse_program(src).expect("should parse");
    match &prog.statements[0] {
        TopLevel::ExternCoproc(block) => {
            assert_eq!(block.items.len(), 2);
            assert!(matches!(block.items[0], CoprocItem::Intrinsic(_)));
            assert!(matches!(block.items[1], CoprocItem::Intrinsic(_)));
        }
        other => panic!("expected ExternCoproc, got {other:?}"),
    }
}

#[test]
fn tc1_parse_insn_item_with_encoding() {
    let src = r#"
extern coproc custom_x {
    @pure insn custom_op(a: Int, b: Int): Int
        encoding ".insn r 0x0B, 0, 0, rd, rs1, rs2" ;
}
"#;
    let prog = parse_program(src).expect("should parse");
    match &prog.statements[0] {
        TopLevel::ExternCoproc(block) => {
            assert_eq!(block.items.len(), 1);
            match &block.items[0] {
                CoprocItem::Insn(i) => {
                    assert_eq!(i.name, "custom_op");
                    assert!(i.encoding.is_some());
                    assert!(i.encoding.as_ref().unwrap().contains("0x0B"));
                }
                other => panic!("expected Insn, got {other:?}"),
            }
        }
        other => panic!("expected ExternCoproc, got {other:?}"),
    }
}

#[test]
fn tc1_parse_coproc_alongside_functions() {
    let src = r#"
fn add(a: Int, b: Int): Int {
    return a + b
}

extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
}

fn mul(a: Int, b: Int): Int {
    return a + b
}
"#;
    let prog = parse_program(src).expect("should parse");
    assert_eq!(prog.statements.len(), 3);
    assert!(matches!(prog.statements[0], TopLevel::Function(_)));
    assert!(matches!(prog.statements[1], TopLevel::ExternCoproc(_)));
    assert!(matches!(prog.statements[2], TopLevel::Function(_)));
}

// ── TC2: Resolution — live/dead ─────────────────────────────────────────────

const RISCV_PATA: &str = r#"
pata v1
gate riscv_vec when arch == "riscv64" && feature_v
family riscv_vec = "riscv"
gate zbb when arch == "riscv64" && feature_zbb
family zbb = "riscv"
"#;

fn riscv64_v_env() -> CoprocEnv {
    CoprocEnv::from_triple("riscv64gc-unknown-none-elf", &["v"])
}

fn aarch64_env() -> CoprocEnv {
    CoprocEnv::from_triple("aarch64-unknown-linux-gnu", &[])
}

#[test]
fn tc2_live_gate_keeps_block() {
    let src = r#"
extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
}
"#;
    let prog = parse_program(src).unwrap();
    let (resolved_prog, _ns) =
        resolve_coproc_blocks(prog, &riscv64_v_env(), Some(RISCV_PATA)).unwrap();

    assert_eq!(resolved_prog.statements.len(), 1);
    match &resolved_prog.statements[0] {
        TopLevel::ExternCoproc(block) => {
            let res = block.resolved.as_ref().expect("should be resolved");
            assert!(res.live);
            assert_eq!(res.family, "riscv");
        }
        other => panic!("expected ExternCoproc, got {other:?}"),
    }
}

#[test]
fn tc2_dead_gate_drops_block() {
    let src = r#"
extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
}
"#;
    let prog = parse_program(src).unwrap();
    let (resolved_prog, _ns) =
        resolve_coproc_blocks(prog, &aarch64_env(), Some(RISCV_PATA)).unwrap();

    // Block should have been dropped — program is now empty.
    assert!(resolved_prog.statements.is_empty(),
        "dead block should be dropped, got {:?}", resolved_prog.statements);
}

#[test]
fn tc2_selective_drop_multiple_blocks() {
    let src = r#"
extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
}
extern coproc zbb {
    @pure intrinsic clz(a: Int): Int ;
}
"#;
    let prog = parse_program(src).unwrap();

    // riscv64 with V but without Zbb: vec lives, zbb dies.
    let env = CoprocEnv::from_triple("riscv64gc-unknown-none-elf", &["v"]);
    let (resolved_prog, _ns) = resolve_coproc_blocks(prog, &env, Some(RISCV_PATA)).unwrap();

    assert_eq!(resolved_prog.statements.len(), 1);
    match &resolved_prog.statements[0] {
        TopLevel::ExternCoproc(block) => assert_eq!(block.gate_name, "riscv_vec"),
        _ => panic!("expected riscv_vec block"),
    }
}

#[test]
fn tc2_unknown_gate_is_error() {
    let src = r#"
extern coproc nonexistent_gate {
    @pure intrinsic foo(a: Int): Int ;
}
"#;
    let prog = parse_program(src).unwrap();
    let result = resolve_coproc_blocks(prog, &riscv64_v_env(), Some(RISCV_PATA));
    assert!(
        matches!(result, Err(JtvError::CoprocResolutionFailed { .. })),
        "unknown gate should error, got {result:?}"
    );
}

// ── TC3: Namespace registration ─────────────────────────────────────────────

#[test]
fn tc3_live_decls_registered_in_namespace() {
    let src = r#"
extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
    @pure intrinsic vsub_i32(a: Int, b: Int): Int ;
}
"#;
    let prog = parse_program(src).unwrap();
    let (_resolved_prog, ns) =
        resolve_coproc_blocks(prog, &riscv64_v_env(), Some(RISCV_PATA)).unwrap();

    assert!(ns.get("vadd_i32").is_some(), "vadd_i32 should be in namespace");
    assert!(ns.get("vsub_i32").is_some(), "vsub_i32 should be in namespace");

    let entry = ns.get("vadd_i32").unwrap();
    assert_eq!(entry.gate_name, "riscv_vec");
    assert_eq!(entry.family, "riscv");
    assert_eq!(entry.param_count, 2);
}

#[test]
fn tc3_dead_decls_not_in_namespace() {
    let src = r#"
extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
}
"#;
    let prog = parse_program(src).unwrap();
    let (_resolved_prog, ns) =
        resolve_coproc_blocks(prog, &aarch64_env(), Some(RISCV_PATA)).unwrap();

    assert!(
        ns.get("vadd_i32").is_none(),
        "dead gate's decls should not appear in namespace"
    );
}

// ── TC4: Call-site phase-boundary error ─────────────────────────────────────

#[test]
fn tc4_call_to_live_coproc_returns_phase_boundary_error() {
    let src = r#"
extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
}
x = vadd_i32(1, 2)
"#;
    let prog = parse_program(src).unwrap();
    let (resolved_prog, ns) =
        resolve_coproc_blocks(prog, &riscv64_v_env(), Some(RISCV_PATA)).unwrap();

    let mut interp = Interpreter::new();
    interp.register_coproc_namespace(ns);

    let result = interp.run(&resolved_prog);
    assert!(
        matches!(result, Err(JtvError::ExternCoprocNotYetLowered { ref gate, ref name })
            if gate == "riscv_vec" && name == "vadd_i32"),
        "expected ExternCoprocNotYetLowered, got {result:?}"
    );
}

// ── TC5: No pata source — all blocks survive ─────────────────────────────────

#[test]
fn tc5_no_pata_source_all_blocks_live() {
    let src = r#"
extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
}
"#;
    let prog = parse_program(src).unwrap();
    // Pass None as pata source → unconditionally live.
    let (resolved_prog, ns) =
        resolve_coproc_blocks(prog, &aarch64_env(), None).unwrap();

    assert_eq!(resolved_prog.statements.len(), 1);
    assert!(ns.get("vadd_i32").is_some());
}
