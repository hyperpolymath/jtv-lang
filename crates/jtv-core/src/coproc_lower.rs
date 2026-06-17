// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Zig FFI + Idris2 ABI lowering for live extern coproc decls (JtV Phase A).
//!
//! After `resolve_coproc_blocks` produces a `CoprocNamespace`, this pass
//! generates three artefacts per surviving gate:
//!
//! | Artefact  | Language | Purpose                                            |
//! |-----------|----------|----------------------------------------------------|
//! | `*.zig`   | Zig      | `pub export fn` stubs (unreachable placeholders)   |
//! | `*.idr`   | Idris2   | `%foreign "C:..."` ABI declarations               |
//! | `*.h`     | C        | `extern "C"` header for cross-language consumers   |
//!
//! # Symbol naming convention
//!
//! `jtv_coproc_{gate}_{fn}` with `-` replaced by `_`.
//! Example: gate `riscv_vec`, fn `vadd_i32` → `jtv_coproc_riscv_vec_vadd_i32`.
//!
//! # Filesystem layout (written by `write_lowered`)
//!
//! ```text
//! ffi/zig/coproc/<gate>.zig
//! src/abi/coproc/<GatePascal>.idr
//! generated/abi/coproc/<gate>.h
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! let gates = lower_namespace(&ns);
//! write_lowered(&gates, std::path::Path::new("."))?;
//! ```

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use crate::ast::{BasicType, TypeAnnotation};
use crate::coproc::{CoprocEntry, CoprocKind, CoprocNamespace};

// ── Type-mapping helpers ──────────────────────────────────────────────────────

fn to_zig_type(t: &TypeAnnotation) -> &'static str {
    match t {
        TypeAnnotation::Basic(BasicType::Int) => "i64",
        TypeAnnotation::Basic(BasicType::Float) => "f64",
        TypeAnnotation::Basic(BasicType::Hex) => "u64",
        TypeAnnotation::Basic(BasicType::Binary) => "u64",
        TypeAnnotation::Basic(BasicType::Rational) => "i64",
        TypeAnnotation::Basic(BasicType::Complex) => "i64",
        TypeAnnotation::Basic(BasicType::Symbolic) => "[*:0]const u8",
        _ => "i64",
    }
}

fn to_c_type(t: &TypeAnnotation) -> &'static str {
    match t {
        TypeAnnotation::Basic(BasicType::Int) => "int64_t",
        TypeAnnotation::Basic(BasicType::Float) => "double",
        TypeAnnotation::Basic(BasicType::Hex) => "uint64_t",
        TypeAnnotation::Basic(BasicType::Binary) => "uint64_t",
        TypeAnnotation::Basic(BasicType::Rational) => "int64_t",
        TypeAnnotation::Basic(BasicType::Complex) => "int64_t",
        TypeAnnotation::Basic(BasicType::Symbolic) => "const char*",
        _ => "int64_t",
    }
}

fn to_idris2_type(t: &TypeAnnotation) -> &'static str {
    match t {
        TypeAnnotation::Basic(BasicType::Int) => "Int",
        TypeAnnotation::Basic(BasicType::Float) => "Double",
        TypeAnnotation::Basic(BasicType::Hex) => "Bits64",
        TypeAnnotation::Basic(BasicType::Binary) => "Bits64",
        TypeAnnotation::Basic(BasicType::Rational) => "Int",
        TypeAnnotation::Basic(BasicType::Complex) => "Int",
        TypeAnnotation::Basic(BasicType::Symbolic) => "String",
        _ => "Int",
    }
}

// ── Symbol + module-name helpers ─────────────────────────────────────────────

/// Canonical C/Zig symbol: `jtv_coproc_<gate>_<fn>`.
pub fn sym(gate: &str, fn_name: &str) -> String {
    let g = gate.replace('-', "_");
    let f = fn_name.replace('-', "_");
    format!("jtv_coproc_{}_{}", g, f)
}

fn gate_ident(gate: &str) -> String {
    gate.replace('-', "_")
}

/// PascalCase gate name for use in Idris2 module paths.
fn gate_pascal(gate: &str) -> String {
    gate.split(['_', '-'])
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Positional parameter names: a, b, c, ... p0, p1, p2, ... for > 26.
fn param_name(idx: usize) -> String {
    if idx < 26 {
        ((b'a' + idx as u8) as char).to_string()
    } else {
        format!("p{}", idx)
    }
}

// ── Lowered artefact ─────────────────────────────────────────────────────────

/// Generated source artefacts for one surviving gate.
#[derive(Debug, Clone)]
pub struct LoweredGate {
    /// Gate name from the `.jtv` `extern coproc` block.
    pub gate_name: String,
    /// ISA family from the `.pata` file (empty string in dev mode).
    pub family: String,
    /// Zig source for `ffi/zig/coproc/<gate>.zig`.
    pub zig_source: String,
    /// Idris2 source for `src/abi/coproc/<GatePascal>.idr`.
    pub idris2_source: String,
    /// C header for `generated/abi/coproc/<gate>.h`.
    pub c_header: String,
}

impl LoweredGate {
    /// Relative path for the Zig stub file.
    pub fn zig_path(&self) -> String {
        format!("ffi/zig/coproc/{}.zig", gate_ident(&self.gate_name))
    }

    /// Relative path for the Idris2 ABI file.
    pub fn idris2_path(&self) -> String {
        format!("src/abi/coproc/{}.idr", gate_pascal(&self.gate_name))
    }

    /// Relative path for the C header file.
    pub fn c_header_path(&self) -> String {
        format!("generated/abi/coproc/{}.h", gate_ident(&self.gate_name))
    }
}

// ── Core lowering logic ───────────────────────────────────────────────────────

/// Lower a `CoprocNamespace` into per-gate source artefacts.
///
/// Entries are grouped by gate name and sorted for deterministic output.
/// Returns one `LoweredGate` per gate, sorted by gate name.
pub fn lower_namespace(ns: &CoprocNamespace) -> Vec<LoweredGate> {
    // Group entries by gate: gate_name → (family, BTreeMap<fn_name, entry>)
    let mut by_gate: BTreeMap<String, (String, BTreeMap<String, &CoprocEntry>)> = BTreeMap::new();
    for (fn_name, entry) in &ns.entries {
        by_gate
            .entry(entry.gate_name.clone())
            .or_insert_with(|| (entry.family.clone(), BTreeMap::new()))
            .1
            .insert(fn_name.clone(), entry);
    }

    by_gate
        .into_iter()
        .map(|(gate_name, (family, fns))| {
            let zig_source = emit_zig(&gate_name, &family, &fns);
            let idris2_source = emit_idris2(&gate_name, &family, &fns);
            let c_header = emit_c_header(&gate_name, &family, &fns);
            LoweredGate {
                gate_name,
                family,
                zig_source,
                idris2_source,
                c_header,
            }
        })
        .collect()
}

/// Write all lowered gate artefacts under `root_dir`.
///
/// Creates `ffi/zig/coproc/`, `src/abi/coproc/`, and `generated/abi/coproc/`
/// subdirectories as needed.
pub fn write_lowered(gates: &[LoweredGate], root_dir: &Path) -> io::Result<()> {
    for gate in gates {
        let zig_path = root_dir.join(gate.zig_path());
        let idr_path = root_dir.join(gate.idris2_path());
        let h_path = root_dir.join(gate.c_header_path());

        if let Some(parent) = zig_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if let Some(parent) = idr_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if let Some(parent) = h_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&zig_path, &gate.zig_source)?;
        std::fs::write(&idr_path, &gate.idris2_source)?;
        std::fs::write(&h_path, &gate.c_header)?;
    }
    Ok(())
}

// ── Zig emission ──────────────────────────────────────────────────────────────

fn emit_zig(gate: &str, family: &str, fns: &BTreeMap<String, &CoprocEntry>) -> String {
    let mut out = String::new();

    out.push_str("// SPDX-License-Identifier: MPL-2.0\n");
    out.push_str("// Auto-generated by `jtv lower` — DO NOT EDIT\n");
    out.push_str(&format!("// Gate: {}  Family: {}\n", gate, family));
    out.push_str("//\n");
    out.push_str("// Replace `unreachable` bodies with platform intrinsic calls.\n");
    out.push('\n');

    for (fn_name, entry) in fns {
        let symbol = sym(gate, fn_name);
        let kind_label = match &entry.kind {
            CoprocKind::Intrinsic => "intrinsic".to_string(),
            CoprocKind::Insn { encoding: None } => "insn".to_string(),
            CoprocKind::Insn {
                encoding: Some(enc),
            } => format!("insn (encoding: {})", enc),
        };

        out.push_str(&format!(
            "/// {}: {} — supply a platform implementation.\n",
            kind_label, fn_name
        ));

        // Build parameter list
        let params: Vec<String> = entry
            .param_types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("{}: {}", param_name(i), to_zig_type(ty)))
            .collect();
        let ret_ty = to_zig_type(&entry.return_type);

        out.push_str(&format!(
            "pub export fn {}({}) {} {{\n",
            symbol,
            params.join(", "),
            ret_ty
        ));
        out.push_str("    unreachable;\n");
        out.push_str("}\n\n");
    }

    out
}

// ── Idris2 emission ───────────────────────────────────────────────────────────

fn emit_idris2(gate: &str, family: &str, fns: &BTreeMap<String, &CoprocEntry>) -> String {
    let mut out = String::new();
    let lib_name = format!("libjtv_coproc_{}", gate_ident(gate));
    let module_name = format!("Jtv.Coproc.{}", gate_pascal(gate));

    out.push_str("-- SPDX-License-Identifier: MPL-2.0\n");
    out.push_str("-- Auto-generated by `jtv lower` — DO NOT EDIT\n");
    out.push_str(&format!("-- Gate: {}  Family: {}\n", gate, family));
    out.push('\n');
    out.push_str(&format!("module {}\n", module_name));
    out.push('\n');
    out.push_str("%default total\n");
    out.push('\n');

    for (fn_name, entry) in fns {
        let symbol = sym(gate, fn_name);

        out.push_str(&format!("%foreign \"C:{},{}\"\n", symbol, lib_name));
        out.push_str("export\n");

        // Build Idris2 arrow type: T1 -> T2 -> ... -> RetTy
        let mut type_sig = String::new();
        for ty in &entry.param_types {
            type_sig.push_str(to_idris2_type(ty));
            type_sig.push_str(" -> ");
        }
        type_sig.push_str(to_idris2_type(&entry.return_type));

        out.push_str(&format!("{} : {}\n\n", symbol, type_sig));
    }

    out
}

// ── C header emission ─────────────────────────────────────────────────────────

fn emit_c_header(gate: &str, family: &str, fns: &BTreeMap<String, &CoprocEntry>) -> String {
    let mut out = String::new();
    let guard = format!("JTV_COPROC_{}_H", gate_ident(gate).to_uppercase());

    out.push_str("/* SPDX-License-Identifier: MPL-2.0 */\n");
    out.push_str("/* Auto-generated by `jtv lower` — DO NOT EDIT */\n");
    out.push_str(&format!("/* Gate: {}  Family: {} */\n", gate, family));
    out.push('\n');
    out.push_str(&format!("#ifndef {}\n", guard));
    out.push_str(&format!("#define {}\n", guard));
    out.push('\n');
    out.push_str("#include <stdint.h>\n");
    out.push('\n');
    out.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n");

    for (fn_name, entry) in fns {
        let symbol = sym(gate, fn_name);
        let ret_ty = to_c_type(&entry.return_type);

        let params: Vec<String> = entry
            .param_types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("{} {}", to_c_type(ty), param_name(i)))
            .collect();

        let params_str = if params.is_empty() {
            "void".to_string()
        } else {
            params.join(", ")
        };

        out.push_str(&format!("{} {}({});\n", ret_ty, symbol, params_str));
    }

    out.push_str("\n#ifdef __cplusplus\n}\n#endif\n\n");
    out.push_str(&format!("#endif /* {} */\n", guard));

    out
}
