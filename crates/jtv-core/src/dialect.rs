// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// ADR-0008: purist vs adulterated dialect — the "purity certificate".
//
// JtV ships as two structurally-distinct dialects: *purist-jtv* (no
// expression-level sugar; subtraction is reverse-only) and *adulterated-jtv*
// (admits the `neg` / `-` sugar — the escape hatch). This module is the
// ADR-0008 D4 purity certificate: a READ-ONLY analysis that stamps a program
// `purist` or `adulterated` by counting expression-level `neg` sugar. It
// changes no semantics — it makes the use of the escape hatch visible and
// accountable (accountability without prohibition).
//
// NB: distinct from `purity.rs`, which is the *function*-purity / Pure-Function
// Rule (`@pure` / `@total`). This is about *dialect* purity.

use crate::ast::*;

/// Which dialect a program is written in (ADR-0008).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    /// No expression-level sugar; subtraction is reverse-only.
    Purist,
    /// Uses the `neg` / `-` sugar.
    Adulterated,
}

/// The ADR-0008 D4 purity certificate a build carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurityCertificate {
    pub dialect: Dialect,
    /// Count of expression-level `neg` sugar uses found in the program.
    pub neg_uses: usize,
}

impl PurityCertificate {
    /// True iff the program uses no sugar (purist-jtv).
    pub fn is_purist(&self) -> bool {
        self.dialect == Dialect::Purist
    }

    /// The stamp string for the build artifact / machine-readable output.
    pub fn stamp(&self) -> String {
        match self.dialect {
            Dialect::Purist => "purist-jtv".to_string(),
            Dialect::Adulterated => format!("adulterated-jtv (neg x{})", self.neg_uses),
        }
    }
}

impl std::fmt::Display for PurityCertificate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.stamp())
    }
}

/// Issue the purity certificate for a program (ADR-0008 D4). Pure analysis: it
/// counts expression-level `neg` sugar and never mutates the program. A program
/// that subtracts only via `reverse` / `reversible` blocks stays *purist* — only
/// expression-level `neg` makes it *adulterated*.
pub fn certify(program: &Program) -> PurityCertificate {
    let neg_uses = count_program(program);
    let dialect = if neg_uses == 0 {
        Dialect::Purist
    } else {
        Dialect::Adulterated
    };
    PurityCertificate { dialect, neg_uses }
}

fn count_program(p: &Program) -> usize {
    p.statements.iter().map(count_toplevel).sum()
}

fn count_toplevel(t: &TopLevel) -> usize {
    match t {
        TopLevel::Module(m) => m.body.iter().map(count_toplevel).sum(),
        TopLevel::Function(f) => f.body.iter().map(count_control_stmt).sum(),
        TopLevel::Control(c) => count_control_stmt(c),
        TopLevel::Import(_) | TopLevel::ExternCoproc(_) => 0,
    }
}

fn count_control_stmt(s: &ControlStmt) -> usize {
    match s {
        ControlStmt::Assignment(a) => count_expr(&a.value),
        ControlStmt::If(i) => count_if(i),
        ControlStmt::While(w) => {
            count_control_expr(&w.condition) + w.body.iter().map(count_control_stmt).sum::<usize>()
        }
        ControlStmt::For(fo) => {
            count_range(&fo.range) + fo.body.iter().map(count_control_stmt).sum::<usize>()
        }
        ControlStmt::Return(opt) => opt.iter().map(count_data_expr).sum(),
        ControlStmt::Print(es) => es.iter().map(count_data_expr).sum(),
        ControlStmt::ReverseBlock(b) => b.body.iter().map(count_reversible).sum(),
        ControlStmt::ReversibleBlock(b) => b.body.iter().map(count_reversible).sum(),
        ControlStmt::ReverseToken(_) | ControlStmt::AbandonToken(_) => 0,
        ControlStmt::Block(ss) => ss.iter().map(count_control_stmt).sum(),
    }
}

fn count_if(i: &IfStmt) -> usize {
    count_control_expr(&i.condition)
        + i.then_branch.iter().map(count_control_stmt).sum::<usize>()
        + i.else_branch
            .iter()
            .flatten()
            .map(count_control_stmt)
            .sum::<usize>()
}

fn count_expr(e: &Expr) -> usize {
    match e {
        Expr::Data(d) => count_data_expr(d),
        Expr::Control(c) => count_control_expr(c),
    }
}

fn count_control_expr(e: &ControlExpr) -> usize {
    match e {
        ControlExpr::Data(d) => count_data_expr(d),
        ControlExpr::Comparison(l, _, r) => count_data_expr(l) + count_data_expr(r),
        ControlExpr::Logical(l, _, r) => count_control_expr(l) + count_control_expr(r),
        ControlExpr::Not(inner) => count_control_expr(inner),
    }
}

fn count_data_expr(e: &DataExpr) -> usize {
    match e {
        // The escape hatch: expression-level negation / subtraction sugar.
        DataExpr::Negate(inner) => 1 + count_data_expr(inner),
        DataExpr::Add(l, r) => count_data_expr(l) + count_data_expr(r),
        DataExpr::FunctionCall(c) => c.args.iter().map(count_data_expr).sum(),
        DataExpr::List(es) | DataExpr::Tuple(es) => es.iter().map(count_data_expr).sum(),
        DataExpr::Number(_) | DataExpr::StringLit(_) | DataExpr::Identifier(_) => 0,
    }
}

fn count_reversible(s: &ReversibleStmt) -> usize {
    match s {
        ReversibleStmt::AddAssign(_, e) | ReversibleStmt::SubAssign(_, e) => count_data_expr(e),
        ReversibleStmt::If(i) => count_if(i),
    }
}

fn count_range(r: &RangeExpr) -> usize {
    count_data_expr(&r.start)
        + count_data_expr(&r.end)
        + r.step.iter().map(|s| count_data_expr(s)).sum::<usize>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prog(stmts: Vec<TopLevel>) -> Program {
        Program { statements: stmts }
    }

    #[test]
    fn empty_program_is_purist() {
        let cert = certify(&prog(vec![]));
        assert_eq!(cert.dialect, Dialect::Purist);
        assert!(cert.is_purist());
        assert_eq!(cert.stamp(), "purist-jtv");
    }

    #[test]
    fn addition_only_is_purist() {
        // x = a + b  (no neg)
        let stmt = TopLevel::Control(ControlStmt::Assignment(Assignment {
            target: "x".to_string(),
            value: Expr::Data(DataExpr::add(
                DataExpr::identifier("a"),
                DataExpr::identifier("b"),
            )),
        }));
        let cert = certify(&prog(vec![stmt]));
        assert_eq!(cert.dialect, Dialect::Purist);
        assert_eq!(cert.neg_uses, 0);
    }

    #[test]
    fn neg_makes_it_adulterated() {
        // x = -(a)  (expression-level neg sugar)
        let stmt = TopLevel::Control(ControlStmt::Assignment(Assignment {
            target: "x".to_string(),
            value: Expr::Data(DataExpr::negate(DataExpr::identifier("a"))),
        }));
        let cert = certify(&prog(vec![stmt]));
        assert_eq!(cert.dialect, Dialect::Adulterated);
        assert!(!cert.is_purist());
        assert_eq!(cert.neg_uses, 1);
        assert_eq!(cert.stamp(), "adulterated-jtv (neg x1)");
    }

    #[test]
    fn nested_neg_counts_all_uses() {
        // x = (-a) + (-(-b))  ->  3 neg uses
        let value = DataExpr::add(
            DataExpr::negate(DataExpr::identifier("a")),
            DataExpr::negate(DataExpr::negate(DataExpr::identifier("b"))),
        );
        let stmt = TopLevel::Control(ControlStmt::Assignment(Assignment {
            target: "x".to_string(),
            value: Expr::Data(value),
        }));
        let cert = certify(&prog(vec![stmt]));
        assert_eq!(cert.dialect, Dialect::Adulterated);
        assert_eq!(cert.neg_uses, 3);
    }

    #[test]
    fn reverse_block_subtraction_stays_purist() {
        // `reverse { x += a }` — subtraction via the reverse mechanism, NOT
        // expression-level neg. Exactly the purist way to subtract (ADR-0008).
        let stmt = TopLevel::Control(ControlStmt::ReverseBlock(ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::identifier("a"),
            )],
        }));
        let cert = certify(&prog(vec![stmt]));
        assert_eq!(cert.dialect, Dialect::Purist);
        assert_eq!(cert.neg_uses, 0);
    }
}
