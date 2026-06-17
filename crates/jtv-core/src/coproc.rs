// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! PataCL resolution pass for `extern coproc` blocks (Phase 2).
//!
//! Implements Steps 1–3 of the JtV/PataCL integration contract:
//!   1. Name-resolution: for each `extern coproc <gate-name>` block,
//!      invoke PataCL with the gate-name and target fact environment.
//!   2. Block-inclusion: drop blocks whose gate evaluated to dead.
//!   3. Function-namespace registration: surviving decls are added to a
//!      `CoprocNamespace` for the interpreter and type-checker to consume.
//!
//! Call-site phase-boundary error (Step 4) lives in `interpreter.rs`;
//! the namespace built here is what triggers it at evaluation time.
//!
//! # Usage
//!
//! ```rust,ignore
//! let env = CoprocEnv::from_triple("riscv64gc-unknown-none-elf", &["v"]);
//! let pata_src = std::fs::read_to_string("coproc/riscv.pata")?;
//! let (program, ns) = resolve_coproc_blocks(program, &env, Some(&pata_src))?;
//! ```

use std::collections::HashMap;

// PataCL gate evaluation is gated behind the `patacl` cargo feature.
// When the feature is OFF (the default — see `Cargo.toml`), the symbols below
// are absent and `resolve_coproc_blocks` falls back to the honest
// "coprocessor disabled" path (no fake gate results).
#[cfg(feature = "patacl")]
use patacl_core::{compile as patacl_compile, env_from_triple, eval::eval_gates, GateResult};

use crate::ast::{CoprocItem, CoprocResolution, ExternCoprocBlock, Program, TopLevel};
use crate::error::{JtvError, Result};

// ──────────────────────────────────────────────
// Coproc environment
// ──────────────────────────────────────────────

/// Build-time environment for PataCL gate evaluation.
///
/// The fields capture the target triple and feature set used to evaluate
/// `.pata` gates. They are only *read* by `patacl_env()`, which exists solely
/// under the `patacl` cargo feature. With that feature off (the default build),
/// nothing consumes them yet — but they are still part of the public
/// constructor contract, so we suppress the dead-code lint rather than drop
/// real data the coprocessor build needs.
pub struct CoprocEnv {
    #[cfg_attr(not(feature = "patacl"), allow(dead_code))]
    target_triple: String,
    #[cfg_attr(not(feature = "patacl"), allow(dead_code))]
    features: Vec<String>,
}

impl CoprocEnv {
    pub fn new(
        target_triple: impl Into<String>,
        features: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        CoprocEnv {
            target_triple: target_triple.into(),
            features: features.into_iter().map(|f| f.into()).collect(),
        }
    }

    /// Convenience constructor — same argument shape as `patacl_core::env_from_triple`.
    pub fn from_triple(triple: &str, features: &[&str]) -> Self {
        CoprocEnv::new(triple, features.iter().copied())
    }

    #[cfg(feature = "patacl")]
    fn patacl_env(&self) -> patacl_core::FactEnv {
        let refs: Vec<&str> = self.features.iter().map(|s| s.as_str()).collect();
        env_from_triple(&self.target_triple, &refs)
    }
}

// ──────────────────────────────────────────────
// Function namespace entry
// ──────────────────────────────────────────────

/// An extern coproc function registered in the function namespace.
/// The interpreter uses this to return `ExternCoprocNotYetLowered` at
/// call sites (per ADR-0005 call-site contract).
/// `param_types` and `return_type` are propagated from the AST for use
/// by the Zig FFI / Idris2 ABI lowering pass.
#[derive(Debug, Clone)]
pub struct CoprocEntry {
    /// Gate name this decl came from (for the phase-boundary error message).
    pub gate_name: String,
    /// ISA family string evaluated from the `.pata` file.
    pub family: String,
    pub kind: CoprocKind,
    pub param_count: usize,
    /// Parameter types in declaration order.
    /// Missing annotations (`Option::None` in the AST) default to `Int`.
    pub param_types: Vec<crate::ast::TypeAnnotation>,
    pub return_type: crate::ast::TypeAnnotation,
}

#[derive(Debug, Clone)]
pub enum CoprocKind {
    Intrinsic,
    Insn { encoding: Option<String> },
}

/// Namespace of surviving extern coproc function entries.
/// Keyed by function name (unqualified); gate_name is in the value.
#[derive(Debug, Clone, Default)]
pub struct CoprocNamespace {
    pub entries: HashMap<String, CoprocEntry>,
}

impl CoprocNamespace {
    pub fn get(&self, name: &str) -> Option<&CoprocEntry> {
        self.entries.get(name)
    }
}

// ──────────────────────────────────────────────
// Resolution pass
// ──────────────────────────────────────────────

/// Run the PataCL resolution pass over a parsed JtV program.
///
/// For each `extern coproc <gate>` block:
///   - If `pata_source` is `Some`, evaluate the gate via PataCL.
///   - If the gate is dead → drop the block.
///   - If the gate is live (or pata_source is None) → annotate the block
///     with `CoprocResolution` and add its decls to the namespace.
///
/// Returns the modified program (dead blocks removed) and the namespace.
pub fn resolve_coproc_blocks(
    program: Program,
    env: &CoprocEnv,
    pata_source: Option<&str>,
) -> Result<(Program, CoprocNamespace)> {
    // Compile the pata file once, if provided, to get all gate results.
    //
    // Gate evaluation requires the `patacl` feature. With the feature OFF,
    // a provided `pata_source` cannot be honoured, so we surface an honest
    // `CoprocResolutionFailed` rather than fabricating gate results. A `None`
    // source still works (all blocks treated as unconditionally live — the
    // pre-pata development path).
    #[cfg(feature = "patacl")]
    let gate_results: HashMap<String, GateResult> = if let Some(src) = pata_source {
        let gates = patacl_compile(src).map_err(|e| JtvError::CoprocResolutionFailed {
            gate: "<pata-file>".into(),
            detail: e.to_string(),
        })?;
        let patacl_env = env.patacl_env();
        eval_gates(&gates, &patacl_env)
            .map_err(|e| JtvError::CoprocResolutionFailed {
                gate: "<pata-file>".into(),
                detail: e.to_string(),
            })?
            .into_iter()
            .map(|r| (r.name.clone(), r))
            .collect()
    } else {
        HashMap::new()
    };

    #[cfg(not(feature = "patacl"))]
    if pata_source.is_some() {
        let _ = env;
        return Err(JtvError::CoprocResolutionFailed {
            gate: "<pata-file>".into(),
            detail: "PataCL coprocessor support is disabled in this build \
                     (the `patacl` cargo feature is OFF); cannot evaluate \
                     gates from a .pata source. Rebuild jtv-core with \
                     `--features patacl` and a resolvable `patacl-core` \
                     dependency to enable gate evaluation."
                .into(),
        });
    }

    let mut namespace = CoprocNamespace::default();
    let mut filtered = Vec::with_capacity(program.statements.len());

    for item in program.statements {
        match item {
            TopLevel::ExternCoproc(block) => {
                #[cfg(feature = "patacl")]
                let (keep, resolved) = resolve_one_block(&block, pata_source, &gate_results)?;
                // Without `patacl`, `pata_source` is guaranteed `None` here
                // (the `Some` case returned above), so every block is live
                // with no PataCL-derived resolution.
                #[cfg(not(feature = "patacl"))]
                let (keep, resolved): (bool, Option<CoprocResolution>) = (true, None);
                if keep {
                    let family = resolved
                        .as_ref()
                        .map(|r| r.family.clone())
                        .unwrap_or_default();
                    register_block_decls(&block, &family, &mut namespace);
                    filtered.push(TopLevel::ExternCoproc(ExternCoprocBlock {
                        resolved,
                        ..block
                    }));
                }
                // Dead blocks are silently dropped.
            }
            other => filtered.push(other),
        }
    }

    Ok((
        Program {
            statements: filtered,
        },
        namespace,
    ))
}

/// Resolve a single block: returns (keep, Option<CoprocResolution>).
#[cfg(feature = "patacl")]
fn resolve_one_block(
    block: &ExternCoprocBlock,
    pata_source: Option<&str>,
    gate_results: &HashMap<String, GateResult>,
) -> Result<(bool, Option<CoprocResolution>)> {
    if pata_source.is_none() {
        // No pata source — treat all blocks as unconditionally live.
        // Useful during development before .pata files are written.
        return Ok((true, None));
    }

    match gate_results.get(&block.gate_name) {
        Some(result) => {
            if result.live {
                Ok((
                    true,
                    Some(CoprocResolution {
                        live: true,
                        family: result.family.clone().unwrap_or_default(),
                    }),
                ))
            } else {
                Ok((false, None))
            }
        }
        None => {
            // Gate name not found in pata file.
            // Per ADR-0004 strict semantics, this is a build error.
            Err(JtvError::CoprocResolutionFailed {
                gate: block.gate_name.clone(),
                detail: format!(
                    "gate `{}` not found in .pata source; \
                     declare it with `gate {} when <predicate>`",
                    block.gate_name, block.gate_name
                ),
            })
        }
    }
}

/// Register all decls in a live block into the namespace.
fn register_block_decls(block: &ExternCoprocBlock, family: &str, ns: &mut CoprocNamespace) {
    use crate::ast::{BasicType, TypeAnnotation};

    // Fall-back type when a param has no annotation: JtV default is Int.
    let default_ty = || TypeAnnotation::Basic(BasicType::Int);

    for item in &block.items {
        match item {
            CoprocItem::Intrinsic(i) => {
                let param_types = i
                    .params
                    .iter()
                    .map(|p| p.type_annotation.clone().unwrap_or_else(default_ty))
                    .collect();
                ns.entries.insert(
                    i.name.clone(),
                    CoprocEntry {
                        gate_name: block.gate_name.clone(),
                        family: family.to_string(),
                        kind: CoprocKind::Intrinsic,
                        param_count: i.params.len(),
                        param_types,
                        return_type: i.return_type.clone(),
                    },
                );
            }
            CoprocItem::Insn(i) => {
                let param_types = i
                    .params
                    .iter()
                    .map(|p| p.type_annotation.clone().unwrap_or_else(default_ty))
                    .collect();
                ns.entries.insert(
                    i.name.clone(),
                    CoprocEntry {
                        gate_name: block.gate_name.clone(),
                        family: family.to_string(),
                        kind: CoprocKind::Insn {
                            encoding: i.encoding.clone(),
                        },
                        param_count: i.params.len(),
                        param_types,
                        return_type: i.return_type.clone(),
                    },
                );
            }
        }
    }
}
