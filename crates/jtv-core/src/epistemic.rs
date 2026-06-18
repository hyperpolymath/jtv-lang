// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// ADR-0009 D2: the Epistemic effect — what a function reveals about its inputs
// (knowledge / observability). Lattice:
//
//     Opaque (reveals nothing) ⊑ Partial (reveals a bounded function of inputs)
//                              ⊑ Transparent (reveals the inputs fully)
//
// Dual to Echo (loss vs revelation) with the same join law. The v1 inference
// here is output-based and deliberately conservative: a function is graded by
// the worst-case revelation across its OUTPUT expressions (each `return e` /
// `print e`). Refine on use, per ADR-0009.

use crate::ast::*;
use std::collections::HashSet;

/// The epistemic grade (ADR-0009 D2). The enum lives in `ast` (so the AST can
/// carry an `@epi(...)` annotation without a module cycle); re-exported here,
/// where its lattice operations are defined.
pub use crate::ast::Epistemic;

impl Epistemic {
    fn rank(self) -> u8 {
        match self {
            Epistemic::Opaque => 0,
            Epistemic::Partial => 1,
            Epistemic::Transparent => 2,
        }
    }

    /// Least upper bound: the worst-case revelation (the higher of the two).
    pub fn join(self, other: Epistemic) -> Epistemic {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }

    /// Lattice order `a ⊑ b`.
    pub fn leq(self, other: Epistemic) -> bool {
        self.rank() <= other.rank()
    }
}

/// The epistemic grade a function induces (ADR-0009 D2) — the worst-case
/// revelation across its output expressions (each `return e` and `print e`). An
/// output that *is* an input parameter is `Transparent`; one that merely
/// references an input is `Partial`; one independent of the inputs is `Opaque`.
/// A function with no outputs is `Opaque`. (Conservative v1; refine on use.)
pub fn function_epistemic(func: &FunctionDecl) -> Epistemic {
    let params: HashSet<&str> = func.params.iter().map(|p| p.name.as_str()).collect();
    let mut outputs: Vec<&DataExpr> = Vec::new();
    collect_outputs(&func.body, &mut outputs);
    outputs
        .iter()
        .map(|&e| output_epistemic(e, &params))
        .fold(Epistemic::Opaque, Epistemic::join)
}

fn output_epistemic(e: &DataExpr, params: &HashSet<&str>) -> Epistemic {
    if is_param(e, params) {
        Epistemic::Transparent
    } else if refs_param(e, params) {
        Epistemic::Partial
    } else {
        Epistemic::Opaque
    }
}

/// `e` is exactly an input parameter.
fn is_param(e: &DataExpr, params: &HashSet<&str>) -> bool {
    matches!(e, DataExpr::Identifier(name) if params.contains(name.as_str()))
}

/// `e` references any input parameter (transitively).
fn refs_param(e: &DataExpr, params: &HashSet<&str>) -> bool {
    match e {
        DataExpr::Identifier(name) => params.contains(name.as_str()),
        DataExpr::Add(l, r) => refs_param(l, params) || refs_param(r, params),
        DataExpr::Negate(inner) => refs_param(inner, params),
        DataExpr::FunctionCall(c) => c.args.iter().any(|a| refs_param(a, params)),
        DataExpr::List(es) | DataExpr::Tuple(es) => es.iter().any(|x| refs_param(x, params)),
        DataExpr::Number(_) | DataExpr::StringLit(_) => false,
    }
}

/// Collect every output expression (`return e` value, `print` args) in a body.
fn collect_outputs<'a>(body: &'a [ControlStmt], out: &mut Vec<&'a DataExpr>) {
    for s in body {
        match s {
            ControlStmt::Return(Some(e)) => out.push(e),
            ControlStmt::Print(es) => out.extend(es.iter()),
            ControlStmt::If(i) => {
                collect_outputs(&i.then_branch, out);
                if let Some(b) = &i.else_branch {
                    collect_outputs(b, out);
                }
            }
            ControlStmt::While(w) => collect_outputs(&w.body, out),
            ControlStmt::For(f) => collect_outputs(&f.body, out),
            ControlStmt::Block(ss) => collect_outputs(ss, out),
            ControlStmt::Return(None)
            | ControlStmt::Assignment(_)
            | ControlStmt::ReverseBlock(_)
            | ControlStmt::ReversibleBlock(_)
            | ControlStmt::ReverseToken(_)
            | ControlStmt::AbandonToken(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn func(params: Vec<&str>, body: Vec<ControlStmt>) -> FunctionDecl {
        FunctionDecl {
            name: "f".to_string(),
            params: params
                .into_iter()
                .map(|n| Param {
                    name: n.to_string(),
                    type_annotation: None,
                })
                .collect(),
            return_type: None,
            purity: Purity::Pure,
            echo_annotation: None,
            epi_annotation: None,
            body,
        }
    }

    #[test]
    fn lattice_join_is_max() {
        use Epistemic::*;
        assert_eq!(Opaque.join(Partial), Partial);
        assert_eq!(Partial.join(Transparent), Transparent);
        assert_eq!(Transparent.join(Opaque), Transparent);
        assert_eq!(Opaque.join(Opaque), Opaque);
        assert!(Opaque.leq(Transparent));
        assert!(!Transparent.leq(Opaque));
    }

    #[test]
    fn returning_a_param_is_transparent() {
        // f(x) { return x }
        let f = func(
            vec!["x"],
            vec![ControlStmt::Return(Some(DataExpr::identifier("x")))],
        );
        assert_eq!(function_epistemic(&f), Epistemic::Transparent);
    }

    #[test]
    fn returning_a_constant_is_opaque() {
        // f(x) { return 1 }
        let f = func(
            vec!["x"],
            vec![ControlStmt::Return(Some(DataExpr::Number(Number::Int(1))))],
        );
        assert_eq!(function_epistemic(&f), Epistemic::Opaque);
    }

    #[test]
    fn returning_a_function_of_input_is_partial() {
        // f(x) { return x + 1 }
        let f = func(
            vec!["x"],
            vec![ControlStmt::Return(Some(DataExpr::add(
                DataExpr::identifier("x"),
                DataExpr::Number(Number::Int(1)),
            )))],
        );
        assert_eq!(function_epistemic(&f), Epistemic::Partial);
    }

    #[test]
    fn no_output_is_opaque() {
        // f(x) { y = 1 }
        let f = func(
            vec!["x"],
            vec![ControlStmt::Assignment(Assignment {
                target: "y".to_string(),
                value: Expr::Data(DataExpr::Number(Number::Int(1))),
            })],
        );
        assert_eq!(function_epistemic(&f), Epistemic::Opaque);
    }

    #[test]
    fn printing_an_input_is_transparent() {
        // f(x) { print x }
        let f = func(
            vec!["x"],
            vec![ControlStmt::Print(vec![DataExpr::identifier("x")])],
        );
        assert_eq!(function_epistemic(&f), Epistemic::Transparent);
    }
}
