// SPDX-License-Identifier: MPL-2.0
// (MPL-2.0 is automatic legal fallback until PMPL is formally recognised)
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Lowering pass tests (Phase A — Zig FFI + Idris2 ABI emission).
//!
//! CL1 — Zig stubs: symbol names, parameter types, unreachable body
//! CL2 — Idris2 ABI: %foreign pragma, module path, arrow types
//! CL3 — C header: include guard, int64_t params, extern "C"
//! CL4 — insn encoding appears in Zig comment
//! CL5 — multi-gate namespace: two separate LoweredGate outputs
//! CL6 — native impl dispatches instead of ExternCoprocNotYetLowered

use jtv_core::{
    coproc::{CoprocEnv, resolve_coproc_blocks},
    coproc_lower::{lower_namespace, sym},
    error::JtvError,
    interpreter::Interpreter,
    parser::parse_program,
};

const RISCV_PATA: &str =
    include_str!("../../../conformance/coproc/riscv.pata");

const LIVE_INTRINSIC_JTV: &str =
    include_str!("../../../conformance/coproc/live_intrinsic.jtv");

const INSN_WITH_ENCODING_JTV: &str =
    include_str!("../../../conformance/coproc/insn_with_encoding.jtv");

const MULTI_BLOCK_SELECTIVE_JTV: &str =
    include_str!("../../../conformance/coproc/multi_block_selective.jtv");

fn riscv64_v() -> CoprocEnv {
    CoprocEnv::from_triple("riscv64gc-unknown-none-elf", &["v"])
}

// ── CL1: Zig stubs ───────────────────────────────────────────────────────────

#[test]
fn cl1_zig_stubs_symbol_names_and_body() {
    let prog = parse_program(LIVE_INTRINSIC_JTV).expect("should parse");
    let (_, ns) =
        resolve_coproc_blocks(prog, &riscv64_v(), Some(RISCV_PATA)).expect("should resolve");

    let gates = lower_namespace(&ns);
    assert_eq!(gates.len(), 1, "one gate expected");

    let gate = &gates[0];
    assert_eq!(gate.gate_name, "riscv_vec");

    let zig = &gate.zig_source;
    // Both symbols present
    assert!(zig.contains("pub export fn jtv_coproc_riscv_vec_vadd_i32("),
        "vadd_i32 stub missing from Zig:\n{}", zig);
    assert!(zig.contains("pub export fn jtv_coproc_riscv_vec_vsub_i32("),
        "vsub_i32 stub missing from Zig:\n{}", zig);

    // Parameters typed as i64 (Int → i64)
    assert!(zig.contains("a: i64, b: i64") || zig.contains("i64"),
        "i64 type expected in Zig params:\n{}", zig);

    // Placeholder body
    assert!(zig.contains("unreachable"),
        "unreachable body expected:\n{}", zig);

    // SPDX header
    assert!(zig.starts_with("// SPDX-License-Identifier: MPL-2.0"),
        "SPDX header missing");
}

// ── CL2: Idris2 ABI ─────────────────────────────────────────────────────────

#[test]
fn cl2_idris2_foreign_pragma_and_module() {
    let prog = parse_program(LIVE_INTRINSIC_JTV).expect("should parse");
    let (_, ns) =
        resolve_coproc_blocks(prog, &riscv64_v(), Some(RISCV_PATA)).expect("should resolve");

    let gates = lower_namespace(&ns);
    let idr = &gates[0].idris2_source;

    // Module declaration
    assert!(idr.contains("module Jtv.Coproc.RiscvVec"),
        "module declaration missing:\n{}", idr);

    // %foreign pragma for vadd_i32
    let vadd_sym = sym("riscv_vec", "vadd_i32");
    assert!(idr.contains(&format!("%foreign \"C:{},", vadd_sym)),
        "%foreign pragma missing for vadd_i32:\n{}", idr);

    // Arrow type: Int -> Int -> Int
    assert!(idr.contains("Int -> Int -> Int"),
        "arrow type missing in Idris2 output:\n{}", idr);

    // export keyword
    assert!(idr.contains("\nexport\n"),
        "export keyword missing:\n{}", idr);
}

// ── CL3: C header ────────────────────────────────────────────────────────────

#[test]
fn cl3_c_header_include_guard_and_types() {
    let prog = parse_program(LIVE_INTRINSIC_JTV).expect("should parse");
    let (_, ns) =
        resolve_coproc_blocks(prog, &riscv64_v(), Some(RISCV_PATA)).expect("should resolve");

    let gates = lower_namespace(&ns);
    let h = &gates[0].c_header;

    // Include guard
    assert!(h.contains("#ifndef JTV_COPROC_RISCV_VEC_H"),
        "include guard missing:\n{}", h);

    // int64_t types and extern "C"
    assert!(h.contains("int64_t"), "int64_t missing:\n{}", h);
    assert!(h.contains("extern \"C\""), "extern \"C\" missing:\n{}", h);
    assert!(h.contains("#include <stdint.h>"), "stdint.h include missing:\n{}", h);

    // Both function declarations
    assert!(h.contains("jtv_coproc_riscv_vec_vadd_i32"),
        "vadd_i32 decl missing:\n{}", h);
    assert!(h.contains("jtv_coproc_riscv_vec_vsub_i32"),
        "vsub_i32 decl missing:\n{}", h);

    // Closing guard
    assert!(h.contains("#endif"), "closing guard missing:\n{}", h);
}

// ── CL4: insn encoding in Zig ────────────────────────────────────────────────

#[test]
fn cl4_insn_encoding_appears_in_zig_comment() {
    let prog = parse_program(INSN_WITH_ENCODING_JTV).expect("should parse");
    let (_, ns) =
        resolve_coproc_blocks(prog, &riscv64_v(), Some(RISCV_PATA)).expect("should resolve");

    let gates = lower_namespace(&ns);
    assert!(!gates.is_empty(), "expected at least one gate");

    let zig = &gates[0].zig_source;
    assert!(zig.contains("0x0B"),
        "encoding '0x0B' should appear in Zig output:\n{}", zig);
}

// ── CL5: multi-gate namespace ─────────────────────────────────────────────────

#[test]
fn cl5_multi_gate_namespace_two_outputs() {
    // multi_block_selective has riscv_vec (live on riscv64+V) and riscv_zbb (dead)
    let prog = parse_program(MULTI_BLOCK_SELECTIVE_JTV).expect("should parse");
    let (_, ns) =
        resolve_coproc_blocks(prog, &riscv64_v(), Some(RISCV_PATA)).expect("should resolve");

    // Only riscv_vec survives (riscv_zbb is dead)
    let gates = lower_namespace(&ns);
    assert_eq!(gates.len(), 1, "only riscv_vec should survive");
    assert_eq!(gates[0].gate_name, "riscv_vec");
}

// ── CL6: native impl dispatch ────────────────────────────────────────────────

#[test]
fn cl6_native_impl_dispatches_instead_of_phase_boundary_error() {
    use jtv_core::number::Value;

    let src = r#"
extern coproc riscv_vec {
    @pure intrinsic vadd_i32(a: Int, b: Int): Int ;
}
result = vadd_i32(10, 32)
"#;
    let prog = parse_program(src).expect("should parse");
    let env = riscv64_v();
    let (resolved, ns) =
        resolve_coproc_blocks(prog, &env, Some(RISCV_PATA)).expect("should resolve");

    let mut interp = Interpreter::new();
    interp.register_coproc_namespace(ns);

    // Register a native impl: just adds the two Int args.
    interp.register_coproc_impl("vadd_i32", |args| {
        match (&args[0], &args[1]) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            _ => Err(JtvError::TypeError("expected Int".into())),
        }
    });

    interp.run(&resolved).expect("should not return phase-boundary error");

    let result = interp.get_variables()
        .into_iter()
        .find(|(k, _)| k == "result")
        .map(|(_, v)| v);

    assert_eq!(result, Some(Value::Int(42)),
        "native impl should compute 10+32=42");
}
