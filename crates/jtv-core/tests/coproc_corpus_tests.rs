// SPDX-License-Identifier: MPL-2.0
// (MPL-2.0 is automatic legal fallback until PMPL is formally recognised)
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Coproc conformance corpus tests (Phase 4 — JtV side).
//!
//! Each test loads a .jtv source file from conformance/coproc/ together with
//! its companion .pata file (or None for dev-mode tests), runs the full
//! resolve_coproc_blocks pipeline, and asserts exact outcomes.
//!
//! CC1 — live_intrinsic.jtv      : live gate, two intrinsics in namespace
//! CC2 — dead_gate_dropped.jtv   : dead gate, block absent, fallback fn present
//! CC3 — multi_block_selective   : vec lives, zbb dies — correct namespace
//! CC4 — no_pata_all_live.jtv    : dev mode, all blocks survive unconditionally
//! CC5 — insn_with_encoding.jtv  : Insn kind + encoding string preserved
//! CC6 — unknown_gate.jtv        : CoprocResolutionFailed on missing gate name
//! CC7 — phase boundary          : ExternCoprocNotYetLowered fires at call site

use jtv_core::{
    ast::TopLevel,
    coproc::{CoprocEnv, CoprocKind, resolve_coproc_blocks},
    error::JtvError,
    interpreter::Interpreter,
    parser::parse_program,
};

// ── Embedded corpus fixtures ─────────────────────────────────────────────────

const RISCV_PATA: &str =
    include_str!("../../../conformance/coproc/riscv.pata");

const LIVE_INTRINSIC_JTV: &str =
    include_str!("../../../conformance/coproc/live_intrinsic.jtv");

const DEAD_GATE_DROPPED_JTV: &str =
    include_str!("../../../conformance/coproc/dead_gate_dropped.jtv");

const MULTI_BLOCK_SELECTIVE_JTV: &str =
    include_str!("../../../conformance/coproc/multi_block_selective.jtv");

const NO_PATA_ALL_LIVE_JTV: &str =
    include_str!("../../../conformance/coproc/no_pata_all_live.jtv");

const INSN_WITH_ENCODING_JTV: &str =
    include_str!("../../../conformance/coproc/insn_with_encoding.jtv");

const UNKNOWN_GATE_JTV: &str =
    include_str!("../../../conformance/coproc/unknown_gate.jtv");

const UNKNOWN_GATE_PATA: &str =
    include_str!("../../../conformance/coproc/unknown_gate.pata");

// ── Helpers ──────────────────────────────────────────────────────────────────

fn riscv64_v() -> CoprocEnv {
    CoprocEnv::from_triple("riscv64gc-unknown-none-elf", &["v"])
}

fn aarch64() -> CoprocEnv {
    CoprocEnv::from_triple("aarch64-unknown-linux-gnu", &[])
}

// ── CC1: live gate — two intrinsics retained ─────────────────────────────────

#[test]
fn cc1_live_gate_two_intrinsics_in_namespace() {
    let prog = parse_program(LIVE_INTRINSIC_JTV).expect("should parse");
    let (resolved, ns) =
        resolve_coproc_blocks(prog, &riscv64_v(), Some(RISCV_PATA)).expect("should resolve");

    // ExternCoproc block is retained; the scalar_add fn is also present.
    assert_eq!(resolved.statements.len(), 2);
    assert!(matches!(resolved.statements[0], TopLevel::ExternCoproc(_)));
    assert!(matches!(resolved.statements[1], TopLevel::Function(_)));

    // Both intrinsics registered.
    let vadd = ns.get("vadd_i32").expect("vadd_i32 should be in namespace");
    assert_eq!(vadd.gate_name, "riscv_vec");
    assert_eq!(vadd.family, "riscv");
    assert_eq!(vadd.param_count, 2);
    assert!(matches!(vadd.kind, CoprocKind::Intrinsic));

    let vsub = ns.get("vsub_i32").expect("vsub_i32 should be in namespace");
    assert_eq!(vsub.param_count, 2);
    assert!(matches!(vsub.kind, CoprocKind::Intrinsic));
}

// ── CC2: dead gate — block absent, function still present ────────────────────

#[test]
fn cc2_dead_gate_block_absent_fn_present() {
    let prog = parse_program(DEAD_GATE_DROPPED_JTV).expect("should parse");
    let (resolved, ns) =
        resolve_coproc_blocks(prog, &aarch64(), Some(RISCV_PATA)).expect("should resolve");

    // Only the fallback fn survives.
    assert_eq!(resolved.statements.len(), 1,
        "dead block should be dropped; got {:?}", resolved.statements);
    assert!(matches!(resolved.statements[0], TopLevel::Function(_)));

    // Nothing in namespace.
    assert!(ns.get("vadd_i32").is_none(),
        "dead gate's decls must not appear in namespace");
}

// ── CC3: selective survival — vec lives, zbb dies ───────────────────────────

#[test]
fn cc3_selective_survival_vec_lives_zbb_dies() {
    let prog = parse_program(MULTI_BLOCK_SELECTIVE_JTV).expect("should parse");
    // riscv64 + V only — no Zbb.
    let (resolved, ns) =
        resolve_coproc_blocks(prog, &riscv64_v(), Some(RISCV_PATA)).expect("should resolve");

    assert_eq!(resolved.statements.len(), 1);
    match &resolved.statements[0] {
        TopLevel::ExternCoproc(b) => assert_eq!(b.gate_name, "riscv_vec"),
        _ => panic!("expected riscv_vec block"),
    }

    // riscv_vec intrinsic present, zbb intrinsics absent.
    assert!(ns.get("vadd_i32").is_some(), "vadd_i32 should survive");
    assert!(ns.get("clz").is_none(),  "clz should be dropped");
    assert!(ns.get("cpop").is_none(), "cpop should be dropped");
}

// ── CC4: dev mode — no pata source, all blocks survive ──────────────────────

#[test]
fn cc4_no_pata_all_blocks_survive() {
    let prog = parse_program(NO_PATA_ALL_LIVE_JTV).expect("should parse");
    // Pass None for pata_source; even on aarch64, block survives.
    let (resolved, ns) =
        resolve_coproc_blocks(prog, &aarch64(), None).expect("should resolve");

    assert_eq!(resolved.statements.len(), 1);
    assert!(matches!(resolved.statements[0], TopLevel::ExternCoproc(_)));

    // Both decls (intrinsic + insn) registered.
    assert!(ns.get("custom_op").is_some(),   "custom_op should be in namespace");
    assert!(ns.get("custom_load").is_some(), "custom_load should be in namespace");
}

// ── CC5: insn item — kind and encoding preserved ─────────────────────────────

#[test]
fn cc5_insn_item_kind_and_encoding_preserved() {
    let prog = parse_program(INSN_WITH_ENCODING_JTV).expect("should parse");
    let (resolved, ns) =
        resolve_coproc_blocks(prog, &riscv64_v(), Some(RISCV_PATA)).expect("should resolve");

    assert_eq!(resolved.statements.len(), 1);

    let entry = ns.get("packed_add").expect("packed_add should be in namespace");
    assert_eq!(entry.gate_name, "riscv_vec");
    assert_eq!(entry.param_count, 2);

    match &entry.kind {
        CoprocKind::Insn { encoding } => {
            let enc = encoding.as_ref().expect("encoding should be present");
            assert!(enc.contains("0x0B"),
                "encoding should contain opcode 0x0B, got {enc:?}");
        }
        _ => panic!("expected CoprocKind::Insn, got {:?}", entry.kind),
    }
}

// ── CC6: unknown gate → CoprocResolutionFailed ──────────────────────────────

#[test]
fn cc6_unknown_gate_is_build_error() {
    let prog = parse_program(UNKNOWN_GATE_JTV).expect("should parse");
    let result = resolve_coproc_blocks(
        prog,
        &riscv64_v(),
        Some(UNKNOWN_GATE_PATA),
    );

    assert!(
        matches!(result, Err(JtvError::CoprocResolutionFailed { ref gate, .. })
            if gate == "mystery_ext"),
        "expected CoprocResolutionFailed for mystery_ext, got {result:?}"
    );
}

// ── CC7: phase boundary — live coproc call raises ExternCoprocNotYetLowered ──

#[test]
fn cc7_live_coproc_call_raises_phase_boundary_error() {
    // Inline source: parse an extern coproc + a call to it.
    let src = r#"
extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
}
result = vadd_i32(1, 2)
"#;
    let prog = parse_program(src).expect("should parse");
    let (resolved, ns) =
        resolve_coproc_blocks(prog, &riscv64_v(), Some(RISCV_PATA)).expect("should resolve");

    let mut interp = Interpreter::new();
    interp.register_coproc_namespace(ns);

    let result = interp.run(&resolved);
    assert!(
        matches!(
            result,
            Err(JtvError::ExternCoprocNotYetLowered { ref gate, ref name })
                if gate == "riscv_vec" && name == "vadd_i32"
        ),
        "expected ExternCoprocNotYetLowered, got {result:?}"
    );
}
