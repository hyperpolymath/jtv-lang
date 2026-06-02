// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Echo: the structured-loss (non-total-erasure) effect lattice for JtV.
//
// This implements the type-checker side of JtV's Echo system (spec v2 §8–9,
// §12) and is the executable counterpart of the formal model in
// `jtv_proofs/JtvEcho.lean`. The taxonomy aligns with the `echo-types` Agda
// library (hyperpolymath/echo-types) and its companion `EchoTypes.jl`.
//
// PRINCIPLE: Echo is about *structured, proof-relevant loss* — information may
// be collapsed, weakened, sampled, projected, or degraded, but the
// residue / provenance / lineage of that loss is still representable. Echo is
// NOT a generic wrapper, a generic Σ-type, or a decorative effect; the object
// of interest is *retained-loss lineage*.
//
//   * `Safe`     — no loss: the operation is injective / reversible
//                  (`+` ↔ `-`). Its fibre over any output is a subsingleton,
//                  so the lineage is trivial.
//   * `Neutral`  — structured loss: information is collapsed, but a residue
//                  carrying the loss lineage/provenance is retained.
//   * `Breaking` — total erasure: lineage is destroyed; not invertible.
//
// Lattice order: `Safe ⊑ Neutral ⊑ Breaking` (join loses guarantees). The
// headline rule, proved as `blockEcho_admissible` in Lean, is that a reverse
// block is admissible iff *no* constituent statement is `Breaking`.
//
// NOTE (spec v2 §12): Echo is an *effect* dimension, independent of value
// typing — it lives alongside `Purity`, not inside `Type`.

use crate::ast::*;

/// The three loss classes of the Echo taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Echo {
    /// No loss — injective / reversible.
    Safe,
    /// Structured loss — non-total erasure, residue retained.
    Neutral,
    /// Total erasure — irreversible.
    Breaking,
}

impl Echo {
    /// Least upper bound. `Breaking` is absorbing; `Safe` is the unit.
    /// Matches `Echo.join` in `JtvEcho.lean`.
    pub fn join(self, other: Echo) -> Echo {
        use Echo::*;
        match (self, other) {
            (Breaking, _) | (_, Breaking) => Breaking,
            (Neutral, _) | (_, Neutral) => Neutral,
            (Safe, Safe) => Safe,
        }
    }

    /// Lattice order `a ≤ b ↔ a ⊔ b = b`.
    pub fn leq(self, other: Echo) -> bool {
        self.join(other) == other
    }

    /// Whether this echo may appear inside a reverse block.
    ///
    /// Policy: **Safe-only.** A reverse block must be fully reversible, so only
    /// `EchoSafe` (bijective `+`/`-`) statements are admissible. `EchoNeutral`
    /// is rejected too: although spec v2 §9 permits it *in principle*
    /// (reversal via a retained residue, Bennett-style), that runtime mechanism
    /// is not implemented, so the checker conservatively requires `Safe`.
    /// `EchoBreaking` is of course always rejected.
    ///
    /// Corresponds to `Echo.admissible` in `JtvEcho.lean`.
    pub fn admissible_in_reverse(self) -> bool {
        self == Echo::Safe
    }
}

impl std::fmt::Display for Echo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Echo::Safe => write!(f, "EchoSafe"),
            Echo::Neutral => write!(f, "EchoNeutral"),
            Echo::Breaking => write!(f, "EchoBreaking"),
        }
    }
}

/// Does `expr` reference variable `var`? Self-reference in a reversible
/// assignment destroys the original value (e.g. `x += x` cannot be inverted),
/// which is exactly a `Breaking` echo.
fn data_expr_uses(expr: &DataExpr, var: &str) -> bool {
    match expr {
        DataExpr::Number(_) | DataExpr::StringLit(_) => false,
        DataExpr::Identifier(name) => name == var,
        DataExpr::Add(l, r) => data_expr_uses(l, var) || data_expr_uses(r, var),
        DataExpr::Negate(inner) => data_expr_uses(inner, var),
        DataExpr::FunctionCall(call) => call.args.iter().any(|a| data_expr_uses(a, var)),
        DataExpr::List(elems) | DataExpr::Tuple(elems) => {
            elems.iter().any(|e| data_expr_uses(e, var))
        }
    }
}

/// Classify the echo of a single reversible statement.
pub fn classify_reversible_stmt(stmt: &ReversibleStmt) -> Echo {
    match stmt {
        // `x += e` / `x -= e` is reversible (Safe) unless the target appears
        // in `e`, in which case the original value is destroyed (Breaking).
        ReversibleStmt::AddAssign(target, expr) | ReversibleStmt::SubAssign(target, expr) => {
            if data_expr_uses(expr, target) {
                Echo::Breaking
            } else {
                Echo::Safe
            }
        }
        // A reversible `if` is as lossy as its lossiest branch. The Data guard
        // is pure (Safe); branches are classified conservatively.
        ReversibleStmt::If(if_stmt) => {
            let then_echo = classify_control_stmts(&if_stmt.then_branch);
            let else_echo = if_stmt
                .else_branch
                .as_ref()
                .map(|b| classify_control_stmts(b))
                .unwrap_or(Echo::Safe);
            then_echo.join(else_echo)
        }
    }
}

/// Aggregate echo of a list of reversible statements: the join of their
/// echoes (starting from `Safe`). Matches `blockEcho` in `JtvEcho.lean`.
pub fn classify_stmts(stmts: &[ReversibleStmt]) -> Echo {
    stmts
        .iter()
        .map(classify_reversible_stmt)
        .fold(Echo::Safe, Echo::join)
}

/// Classify control statements appearing inside a reversible `if` branch.
/// Plain assignments are reversible (Safe); nested reverse blocks recurse;
/// anything else is treated conservatively as `Neutral` (structured loss we
/// cannot yet prove reversible).
fn classify_control_stmts(stmts: &[ControlStmt]) -> Echo {
    stmts
        .iter()
        .map(|s| match s {
            ControlStmt::Assignment(_) => Echo::Safe,
            ControlStmt::ReverseBlock(b) => classify_stmts(&b.body),
            _ => Echo::Neutral,
        })
        .fold(Echo::Safe, Echo::join)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_is_lattice() {
        use Echo::*;
        // breaking is absorbing, safe is the unit, idempotent.
        assert_eq!(Safe.join(Safe), Safe);
        assert_eq!(Safe.join(Neutral), Neutral);
        assert_eq!(Neutral.join(Breaking), Breaking);
        assert_eq!(Breaking.join(Safe), Breaking);
        assert_eq!(Neutral.join(Neutral), Neutral);
        // commutativity on a sample
        assert_eq!(Safe.join(Breaking), Breaking.join(Safe));
    }

    #[test]
    fn order_and_admissibility() {
        use Echo::*;
        assert!(Safe.leq(Neutral));
        assert!(Neutral.leq(Breaking));
        assert!(Safe.leq(Breaking));
        assert!(!Breaking.leq(Safe));
        // Safe-only reversal policy: only Safe is admissible in a reverse block.
        assert!(Safe.admissible_in_reverse());
        assert!(!Neutral.admissible_in_reverse());
        assert!(!Breaking.admissible_in_reverse());
    }

    #[test]
    fn add_assign_independent_is_safe() {
        // x += y  (y independent of x)  ->  Safe
        let stmt =
            ReversibleStmt::AddAssign("x".to_string(), DataExpr::Identifier("y".to_string()));
        assert_eq!(classify_reversible_stmt(&stmt), Echo::Safe);
    }

    #[test]
    fn self_reference_is_breaking() {
        // x += x  destroys the original x  ->  Breaking
        let stmt =
            ReversibleStmt::AddAssign("x".to_string(), DataExpr::Identifier("x".to_string()));
        assert_eq!(classify_reversible_stmt(&stmt), Echo::Breaking);
    }

    #[test]
    fn block_breaking_iff_any_breaking() {
        // [Safe, Safe] -> Safe ; one Breaking poisons the block.
        let safe = ReversibleStmt::AddAssign("x".to_string(), DataExpr::Number(Number::Int(5)));
        let breaking =
            ReversibleStmt::AddAssign("y".to_string(), DataExpr::Identifier("y".to_string()));
        assert_eq!(classify_stmts(&[safe.clone()]), Echo::Safe);
        assert_eq!(classify_stmts(&[safe.clone(), breaking]), Echo::Breaking);
    }
}
