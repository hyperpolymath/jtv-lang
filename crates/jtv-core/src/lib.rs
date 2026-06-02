// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Julia the Viper - Core Language Implementation
// Harvard Architecture: Control (Turing-complete) + Data (Total)

// Allow some clippy lints that require significant refactoring
#![allow(clippy::should_implement_trait)]
#![allow(clippy::while_let_on_iterator)]
#![allow(clippy::manual_strip)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::only_used_in_recursion)]
#![forbid(unsafe_code)]
pub mod ast;
pub mod coproc;
pub mod coproc_lower;
pub mod echo;
pub mod error;
pub mod formatter;
pub mod interpreter;
pub mod number;
pub mod parser;
pub mod pretty;
pub mod purity;
pub mod recovery;
pub mod reversible;
pub mod typechecker;
pub mod wasm;

pub use ast::*;
pub use error::*;
pub use formatter::*;
pub use interpreter::*;
pub use number::*;
pub use parser::*;
pub use pretty::*;
pub use purity::*;
pub use recovery::*;
pub use reversible::*;
pub use typechecker::*;

// WASM bindings are only exported on wasm32, but the analysis helpers
// (AnalysisReport, analyze_code) are available on all targets.
pub use wasm::{analyze_code, AnalysisReport};

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
