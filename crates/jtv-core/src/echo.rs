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
use std::collections::HashMap;

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

    /// Whether this echo may appear inside a plain `reverse { }` block.
    ///
    /// Policy: **Safe-only.** A `reverse { }` block inverts immediately with no
    /// retained residue, so only `EchoSafe` (bijective `+`/`-`) statements are
    /// admissible. `EchoNeutral` is rejected here because, without a token, its
    /// loss lineage is not available to invert from; `EchoBreaking` is of
    /// course always rejected.
    ///
    /// Corresponds to `Echo.admissible` in `JtvEcho.lean`.
    pub fn admissible_in_reverse(self) -> bool {
        self == Echo::Safe
    }

    /// Whether this echo may appear inside a `reversible { } -> tok` block —
    /// the **residue-retaining** (Bennett) policy.
    ///
    /// A `reversible { } -> tok` form records a reversal log and binds a token,
    /// so a later `reverse tok` can invert `EchoNeutral` (structured-loss)
    /// statements by restoring their retained residue — not just `EchoSafe`
    /// ones. `EchoBreaking` (total erasure) is still rejected: no token can
    /// recover destroyed lineage.
    ///
    /// Corresponds to `Echo.admissibleWithResidue` in `JtvEcho.lean`; the
    /// operational justification is `rev_forward_backward_with_token` in
    /// `JtvTheorems`.
    pub fn admissible_with_residue(self) -> bool {
        self != Echo::Breaking
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

// ============================================================================
// CARRIER STRATIFICATION (mirror of `JtvEcho.lean` SECTION 6)
// ============================================================================

/// The additive-algebra class of a carrier — the Rust mirror of `NumAlgebra`
/// in `jtv_proofs/JtvEcho.lean` (SECTION 6). The reversal tier of `+` over a
/// number system is *forced* by this class, not stipulated per system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumAlgebra {
    /// Exact inverses (ℤ, ℚ, ℂ, symbolic, and the ℤ-encodings hex/binary):
    /// reverse-add is total and exact.
    AbelianGroup,
    /// Non-associative / rounding (IEEE-754 float): reverse-add is lossy.
    ApproxGroup,
    /// `+` not invertible (reserved for a future tropical / min-plus system):
    /// no reverse exists.
    NonGroup,
}

/// Where a carrier sits in the additive-algebra tower. `Hex`/`Binary` share
/// `Int`'s class because they are *encodings of ℤ*, not new algebras
/// (`hex_binary_collapse` in Lean). Non-numeric carriers (`Bool`/`String`)
/// have no additive algebra — they cannot be `+=` targets in well-typed code.
pub fn additive_algebra(ty: &BasicType) -> Option<NumAlgebra> {
    use BasicType::*;
    match ty {
        Int | Hex | Binary | Rational | Complex | Symbolic => Some(NumAlgebra::AbelianGroup),
        Float => Some(NumAlgebra::ApproxGroup),
        Bool | String => None,
    }
}

/// The Echo tier *forced* by a carrier's additive algebra — the operational
/// counterpart of `NumSystem.echo` ∘ `NumAlgebra.echo` in Lean SECTION 6:
/// `AbelianGroup → Safe`, `ApproxGroup → Neutral`, `NonGroup → Breaking`. A
/// non-numeric carrier induces no additive-reversal obligation, hence `Safe`.
pub fn carrier_echo(ty: &BasicType) -> Echo {
    match additive_algebra(ty) {
        Some(NumAlgebra::AbelianGroup) | None => Echo::Safe,
        Some(NumAlgebra::ApproxGroup) => Echo::Neutral,
        Some(NumAlgebra::NonGroup) => Echo::Breaking,
    }
}

/// A carrier environment: the declared number system of in-scope variables,
/// built from type annotations (function params today; inferred local types in
/// a later slice). Threaded into Echo classification so that `+=` over a lossy
/// carrier (e.g. float) is graded by the carrier, not just the statement shape.
pub type CarrierEnv = HashMap<String, BasicType>;

/// The carrier echo of a variable. A variable absent from the env is treated as
/// JtV's default numeric carrier `Int` (an exact abelian group → `Safe`): JtV
/// numeric literals default to `int` (cf. `inferType (lit _) = int` in Lean), so
/// an unannotated numeric *is* `int`. The default is therefore sound — only an
/// explicitly-`float` carrier (recorded in the env) lifts a `+=` to `Neutral`.
fn carrier_echo_of(env: &CarrierEnv, var: &str) -> Echo {
    match env.get(var) {
        Some(ty) => carrier_echo(ty),
        None => Echo::Safe,
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

/// Classify the echo of a single reversible statement under a carrier env.
///
/// Two independent sources of loss are joined:
///   1. statement *shape* — self-reference (`x += x`) destroys the original
///      value, recoverable only from a retained residue/token (`Neutral`);
///   2. the *carrier* — over a non-group / approximate number system (e.g.
///      `float`) reverse-add is itself lossy, so the carrier lifts the grade
///      regardless of shape (`carrier_echo`, mirroring Lean SECTION 6).
pub fn classify_reversible_stmt_in_env(stmt: &ReversibleStmt, env: &CarrierEnv) -> Echo {
    match stmt {
        ReversibleStmt::AddAssign(target, expr) | ReversibleStmt::SubAssign(target, expr) => {
            // Shape: self-reference is `Neutral` (token-recoverable, Bennett),
            // never `Breaking` — in the addition-only group every overwrite can
            // be tokenised. Formal basis: `rev_forward_backward_with_token` /
            // `rev_backward_naive_fails_self_ref` in `JtvTheorems`.
            let shape = if data_expr_uses(expr, target) {
                Echo::Neutral
            } else {
                Echo::Safe
            };
            // Carrier: a lossy number system (float) grades the reverse-add up.
            shape.join(carrier_echo_of(env, target))
        }
        // A reversible `if` is as lossy as its lossiest branch. The Data guard
        // is pure (Safe); branches are classified conservatively.
        ReversibleStmt::If(if_stmt) => {
            let then_echo = classify_control_stmts_in_env(&if_stmt.then_branch, env);
            let else_echo = match &if_stmt.else_branch {
                Some(b) => classify_control_stmts_in_env(b, env),
                None => Echo::Safe,
            };
            then_echo.join(else_echo)
        }
    }
}

/// Aggregate echo of reversible statements under a carrier env: the join of
/// their echoes (from `Safe`). Matches `blockEcho` in `JtvEcho.lean`.
pub fn classify_stmts_in_env(stmts: &[ReversibleStmt], env: &CarrierEnv) -> Echo {
    stmts
        .iter()
        .map(|s| classify_reversible_stmt_in_env(s, env))
        .fold(Echo::Safe, Echo::join)
}

/// Classify control statements inside a reversible `if` branch under a carrier
/// env. Plain assignments are reversible (Safe); nested reverse blocks recurse
/// (carrying the env on so their `+=` carriers are seen); anything else is
/// treated conservatively as `Neutral`.
fn classify_control_stmts_in_env(stmts: &[ControlStmt], env: &CarrierEnv) -> Echo {
    stmts
        .iter()
        .map(|s| match s {
            ControlStmt::Assignment(_) => Echo::Safe,
            ControlStmt::ReverseBlock(b) => classify_stmts_in_env(&b.body, env),
            _ => Echo::Neutral,
        })
        .fold(Echo::Safe, Echo::join)
}

/// Shape-only classification of a single reversible statement (no carrier
/// context): every carrier is treated as JtV's default `Int` (Safe). Equivalent
/// to `classify_reversible_stmt_in_env` with an empty env; retained for
/// classifying isolated snippets without a type environment.
pub fn classify_reversible_stmt(stmt: &ReversibleStmt) -> Echo {
    classify_reversible_stmt_in_env(stmt, &CarrierEnv::new())
}

/// Shape-only aggregate echo of reversible statements (empty carrier env).
pub fn classify_stmts(stmts: &[ReversibleStmt]) -> Echo {
    classify_stmts_in_env(stmts, &CarrierEnv::new())
}

// ============================================================================
// SECTION 4: ECHO AS A FUNCTION EFFECT (ADR-0009 D1, slice 1)
// ============================================================================

/// The Echo grade a *function body* induces (ADR-0009 D1) — the inference half
/// of Echo-as-a-first-class-function-effect — under a carrier env. It is the
/// join of the echoes of the body's statements: addition-only data assignments
/// are `Safe` (no loss); `reverse` / `reversible` blocks contribute their block
/// echo (`classify_stmts_in_env`, so float-carrier `+=` grades `Neutral`);
/// control flow joins its sub-bodies.
///
/// Computes a function's *own* body grade; resolving the grade of
/// `FunctionCall`s (joining the callee's into the caller's) is `resolved_effects`
/// in `effect.rs`.
pub fn function_echo_in_env(body: &[ControlStmt], env: &CarrierEnv) -> Echo {
    body.iter()
        .map(|s| control_stmt_echo_in_env(s, env))
        .fold(Echo::Safe, Echo::join)
}

fn control_stmt_echo_in_env(s: &ControlStmt, env: &CarrierEnv) -> Echo {
    match s {
        // Addition-only data assignment: no loss.
        ControlStmt::Assignment(_) => Echo::Safe,
        ControlStmt::If(i) => {
            let then_echo = function_echo_in_env(&i.then_branch, env);
            match &i.else_branch {
                Some(b) => then_echo.join(function_echo_in_env(b, env)),
                None => then_echo,
            }
        }
        ControlStmt::While(w) => function_echo_in_env(&w.body, env),
        ControlStmt::For(f) => function_echo_in_env(&f.body, env),
        // Reverse / reversible blocks carry their own block echo (carrier-aware).
        ControlStmt::ReverseBlock(b) => classify_stmts_in_env(&b.body, env),
        ControlStmt::ReversibleBlock(b) => classify_stmts_in_env(&b.body, env),
        ControlStmt::Block(ss) => function_echo_in_env(ss, env),
        // Reading, printing, and token consumption induce no data loss here.
        ControlStmt::Return(_)
        | ControlStmt::Print(_)
        | ControlStmt::ReverseToken(_)
        | ControlStmt::AbandonToken(_) => Echo::Safe,
    }
}

/// Shape-only function-body echo (no carrier context): every carrier defaults to
/// JtV's `Int` (Safe). Equivalent to `function_echo_in_env` with an empty env.
pub fn function_echo(body: &[ControlStmt]) -> Echo {
    function_echo_in_env(body, &CarrierEnv::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_function_is_safe() {
        assert_eq!(function_echo(&[]), Echo::Safe);
    }

    #[test]
    fn function_echo_addition_only_is_safe() {
        // body: x = a + b
        let body = vec![ControlStmt::Assignment(Assignment {
            target: "x".to_string(),
            value: Expr::Data(DataExpr::add(
                DataExpr::identifier("a"),
                DataExpr::identifier("b"),
            )),
        })];
        assert_eq!(function_echo(&body), Echo::Safe);
    }

    #[test]
    fn function_echo_self_reference_reverse_is_neutral() {
        // body: reverse { x += x } — the block is Neutral, so the function is.
        let body = vec![ControlStmt::ReverseBlock(ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::Identifier("x".to_string()),
            )],
        })];
        assert_eq!(function_echo(&body), Echo::Neutral);
    }

    #[test]
    fn function_echo_joins_branches() {
        // if cond { reverse { x += x } } else { y = 1 }  ->  Neutral join Safe = Neutral
        let then_branch = vec![ControlStmt::ReverseBlock(ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::Identifier("x".to_string()),
            )],
        })];
        let else_branch = vec![ControlStmt::Assignment(Assignment {
            target: "y".to_string(),
            value: Expr::Data(DataExpr::Number(Number::Int(1))),
        })];
        let body = vec![ControlStmt::If(IfStmt {
            condition: ControlExpr::Data(DataExpr::Identifier("cond".to_string())),
            then_branch,
            else_branch: Some(else_branch),
        })];
        assert_eq!(function_echo(&body), Echo::Neutral);
    }

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
        // Safe-only reversal policy (`reverse { }`): only Safe is admissible.
        assert!(Safe.admissible_in_reverse());
        assert!(!Neutral.admissible_in_reverse());
        assert!(!Breaking.admissible_in_reverse());
        // Residue policy (`reversible { } -> tok`): Safe + Neutral admissible,
        // Breaking rejected. Matches `Echo.admissibleWithResidue` in Lean.
        assert!(Safe.admissible_with_residue());
        assert!(Neutral.admissible_with_residue());
        assert!(!Breaking.admissible_with_residue());
    }

    #[test]
    fn add_assign_independent_is_safe() {
        // x += y  (y independent of x)  ->  Safe
        let stmt =
            ReversibleStmt::AddAssign("x".to_string(), DataExpr::Identifier("y".to_string()));
        assert_eq!(classify_reversible_stmt(&stmt), Echo::Safe);
    }

    #[test]
    fn self_reference_is_neutral() {
        // x += x  is lossy but token-recoverable (Bennett)  ->  Neutral, not
        // Breaking. Rejected by `reverse { }` (Safe-only) yet admitted by
        // `reversible { } -> tok` (residue policy).
        let stmt =
            ReversibleStmt::AddAssign("x".to_string(), DataExpr::Identifier("x".to_string()));
        assert_eq!(classify_reversible_stmt(&stmt), Echo::Neutral);
        assert!(!classify_reversible_stmt(&stmt).admissible_in_reverse());
        assert!(classify_reversible_stmt(&stmt).admissible_with_residue());
    }

    #[test]
    fn block_neutral_when_any_self_reference() {
        // [Safe] -> Safe ; a self-referential (Neutral) statement lifts the
        // whole block to Neutral (still token-recoverable, never Breaking).
        let safe = ReversibleStmt::AddAssign("x".to_string(), DataExpr::Number(Number::Int(5)));
        let neutral =
            ReversibleStmt::AddAssign("y".to_string(), DataExpr::Identifier("y".to_string()));
        assert_eq!(classify_stmts(std::slice::from_ref(&safe)), Echo::Safe);
        assert_eq!(classify_stmts(&[safe.clone(), neutral]), Echo::Neutral);
    }

    #[test]
    fn carrier_echo_mirrors_section6() {
        use BasicType::*;
        // Exact abelian groups (incl. the ℤ-encodings hex/binary) are Safe.
        for t in [Int, Hex, Binary, Rational, Complex, Symbolic] {
            assert_eq!(carrier_echo(&t), Echo::Safe);
        }
        // Float is the one approx-group carrier -> Neutral.
        assert_eq!(carrier_echo(&Float), Echo::Neutral);
        // hex/binary collapse onto int's tier exactly (encoding != algebra).
        assert_eq!(carrier_echo(&Hex), carrier_echo(&Int));
        assert_eq!(carrier_echo(&Binary), carrier_echo(&Int));
        // Non-numeric carriers have no additive algebra.
        assert_eq!(additive_algebra(&Bool), None);
        assert_eq!(additive_algebra(&BasicType::String), None);
    }

    #[test]
    fn float_add_assign_is_neutral_via_carrier() {
        // x += y  (y independent of x): Safe over int, Neutral over float.
        let stmt =
            ReversibleStmt::AddAssign("x".to_string(), DataExpr::Identifier("y".to_string()));
        // No carrier context -> default int -> Safe (backward compatible).
        assert_eq!(classify_reversible_stmt(&stmt), Echo::Safe);
        let float_env = CarrierEnv::from([("x".to_string(), BasicType::Float)]);
        assert_eq!(
            classify_reversible_stmt_in_env(&stmt, &float_env),
            Echo::Neutral
        );
        let int_env = CarrierEnv::from([("x".to_string(), BasicType::Int)]);
        assert_eq!(classify_reversible_stmt_in_env(&stmt, &int_env), Echo::Safe);
    }

    #[test]
    fn float_carrier_lifts_block_and_function() {
        // reverse { x += y } over a float x  ->  Neutral (carrier), though the
        // statement shape alone (no self-ref) would be Safe.
        let body = vec![ControlStmt::ReverseBlock(ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::Identifier("y".to_string()),
            )],
        })];
        let float_env = CarrierEnv::from([("x".to_string(), BasicType::Float)]);
        assert_eq!(function_echo_in_env(&body, &float_env), Echo::Neutral);
        // Without carrier context, default int -> Safe.
        assert_eq!(function_echo(&body), Echo::Safe);
    }
}
