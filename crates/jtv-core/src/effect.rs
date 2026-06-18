// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// ADR-0009 D1 + D3: the function effect *row* `(echo, epi)` and its composition
// across calls. `own_effect` reads a function's body (Echo via `function_echo`,
// Epistemic via `function_epistemic`); `resolved_effects` joins in the effects
// of the functions it calls, transitively, to a fixpoint over the call graph.
//
// This is the *composition* half of Echo+Epistemic-as-function-effects. The
// surface `@echo(...)` annotation on a `FunctionDecl` and its upper-bound check
// (`inferred ⊑ annotated`, ADR-0009 D1) landed via the typechecker's
// `check_echo_annotations`, which consumes `resolved_effects` here. Carrying the
// grade in the *type* AST (`TypeAnnotation::Function`, for function-valued
// params) and the parallel `@epi(...)` surface remain later slices.

use crate::ast::*;
use crate::echo::{function_echo_in_env, CarrierEnv, Echo};
use crate::epistemic::{function_epistemic, Epistemic};
use std::collections::HashMap;

/// The graded effect a function carries: a point in the product lattice
/// `Echo × Epistemic` (ADR-0009 D3). Composed componentwise by join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionEffect {
    pub echo: Echo,
    pub epi: Epistemic,
}

impl FunctionEffect {
    /// The bottom of the effect lattice: no loss, reveals nothing.
    pub const SAFE: FunctionEffect = FunctionEffect {
        echo: Echo::Safe,
        epi: Epistemic::Opaque,
    };

    /// Componentwise join of two effects.
    pub fn join(self, other: FunctionEffect) -> FunctionEffect {
        FunctionEffect {
            echo: self.echo.join(other.echo),
            epi: self.epi.join(other.epi),
        }
    }
}

/// A function's OWN effect, from its body alone (not yet its callees').
///
/// The Echo half is *carrier-aware*: parameters with a numeric type annotation
/// seed a `CarrierEnv`, so a `reverse { x += v }` over a `float` parameter grades
/// `Neutral` (lossy reverse-add) rather than `Safe`. See `echo::carrier_echo` and
/// `JtvEcho.lean` SECTION 6.
pub fn own_effect(func: &FunctionDecl) -> FunctionEffect {
    FunctionEffect {
        echo: function_echo_in_env(&func.body, &param_carrier_env(func)),
        epi: function_epistemic(func),
    }
}

/// Seed a carrier environment from a function's parameter type annotations.
/// Locals without annotations default to JtV's `Int` carrier (`Safe`); a fuller
/// inferred-type env is a later slice.
fn param_carrier_env(func: &FunctionDecl) -> CarrierEnv {
    func.params
        .iter()
        .filter_map(|p| match &p.type_annotation {
            Some(TypeAnnotation::Basic(bt)) => Some((p.name.clone(), bt.clone())),
            _ => None,
        })
        .collect()
}

/// Resolve every function's full effect, joining in the effects of the functions
/// it calls (transitively). ADR-0009 D1+D3: composition is join across the call
/// graph. The effect lattice is finite and join is monotone, so the fixpoint
/// terminates — recursion and mutual recursion included.
pub fn resolved_effects(program: &Program) -> HashMap<String, FunctionEffect> {
    let funcs = collect_functions(program);
    let mut env: HashMap<String, FunctionEffect> = funcs
        .iter()
        .map(|f| (f.name.clone(), own_effect(f)))
        .collect();
    let callees: HashMap<String, Vec<String>> = funcs
        .iter()
        .map(|f| (f.name.clone(), body_callees(&f.body)))
        .collect();

    loop {
        let mut changed = false;
        for f in &funcs {
            let mut eff = env[&f.name];
            for c in &callees[&f.name] {
                if let Some(&ce) = env.get(c) {
                    eff = eff.join(ce);
                }
            }
            if eff != env[&f.name] {
                env.insert(f.name.clone(), eff);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    env
}

// ----- traversal: collect declared functions and the calls in a body -----

fn collect_functions(program: &Program) -> Vec<&FunctionDecl> {
    let mut out = Vec::new();
    collect_functions_top(&program.statements, &mut out);
    out
}

fn collect_functions_top<'a>(items: &'a [TopLevel], out: &mut Vec<&'a FunctionDecl>) {
    for t in items {
        match t {
            TopLevel::Function(f) => out.push(f),
            TopLevel::Module(m) => collect_functions_top(&m.body, out),
            _ => {}
        }
    }
}

/// Names of the functions called anywhere in a body.
fn body_callees(body: &[ControlStmt]) -> Vec<String> {
    let mut out = Vec::new();
    for s in body {
        stmt_callees(s, &mut out);
    }
    out
}

fn stmt_callees(s: &ControlStmt, out: &mut Vec<String>) {
    match s {
        ControlStmt::Assignment(a) => expr_callees(&a.value, out),
        ControlStmt::If(i) => {
            control_expr_callees(&i.condition, out);
            for s in &i.then_branch {
                stmt_callees(s, out);
            }
            if let Some(b) = &i.else_branch {
                for s in b {
                    stmt_callees(s, out);
                }
            }
        }
        ControlStmt::While(w) => {
            control_expr_callees(&w.condition, out);
            for s in &w.body {
                stmt_callees(s, out);
            }
        }
        ControlStmt::For(f) => {
            range_callees(&f.range, out);
            for s in &f.body {
                stmt_callees(s, out);
            }
        }
        ControlStmt::Return(Some(e)) => data_callees(e, out),
        ControlStmt::Print(es) => {
            for e in es {
                data_callees(e, out);
            }
        }
        ControlStmt::ReverseBlock(b) => {
            for s in &b.body {
                reversible_callees(s, out);
            }
        }
        ControlStmt::ReversibleBlock(b) => {
            for s in &b.body {
                reversible_callees(s, out);
            }
        }
        ControlStmt::Block(ss) => {
            for s in ss {
                stmt_callees(s, out);
            }
        }
        ControlStmt::Return(None) | ControlStmt::ReverseToken(_) | ControlStmt::AbandonToken(_) => {
        }
    }
}

fn reversible_callees(s: &ReversibleStmt, out: &mut Vec<String>) {
    match s {
        ReversibleStmt::AddAssign(_, e) | ReversibleStmt::SubAssign(_, e) => data_callees(e, out),
        ReversibleStmt::If(i) => {
            control_expr_callees(&i.condition, out);
            for s in &i.then_branch {
                stmt_callees(s, out);
            }
            if let Some(b) = &i.else_branch {
                for s in b {
                    stmt_callees(s, out);
                }
            }
        }
    }
}

fn expr_callees(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Data(d) => data_callees(d, out),
        Expr::Control(c) => control_expr_callees(c, out),
    }
}

fn control_expr_callees(e: &ControlExpr, out: &mut Vec<String>) {
    match e {
        ControlExpr::Data(d) => data_callees(d, out),
        ControlExpr::Comparison(l, _, r) => {
            data_callees(l, out);
            data_callees(r, out);
        }
        ControlExpr::Logical(l, _, r) => {
            control_expr_callees(l, out);
            control_expr_callees(r, out);
        }
        ControlExpr::Not(inner) => control_expr_callees(inner, out),
    }
}

fn data_callees(e: &DataExpr, out: &mut Vec<String>) {
    match e {
        DataExpr::FunctionCall(c) => {
            out.push(c.name.clone());
            for a in &c.args {
                data_callees(a, out);
            }
        }
        DataExpr::Add(l, r) => {
            data_callees(l, out);
            data_callees(r, out);
        }
        DataExpr::Negate(inner) => data_callees(inner, out),
        DataExpr::List(es) | DataExpr::Tuple(es) => {
            for x in es {
                data_callees(x, out);
            }
        }
        DataExpr::Number(_) | DataExpr::StringLit(_) | DataExpr::Identifier(_) => {}
    }
}

fn range_callees(r: &RangeExpr, out: &mut Vec<String>) {
    data_callees(&r.start, out);
    data_callees(&r.end, out);
    if let Some(s) = &r.step {
        data_callees(s, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn func(name: &str, params: Vec<&str>, body: Vec<ControlStmt>) -> FunctionDecl {
        FunctionDecl {
            name: name.to_string(),
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
            body,
        }
    }

    fn call(name: &str, args: Vec<DataExpr>) -> DataExpr {
        DataExpr::FunctionCall(FunctionCall {
            module: None,
            name: name.to_string(),
            args,
        })
    }

    fn assign(target: &str, value: DataExpr) -> ControlStmt {
        ControlStmt::Assignment(Assignment {
            target: target.to_string(),
            value: Expr::Data(value),
        })
    }

    #[test]
    fn no_calls_resolves_to_own_effect() {
        // f(x) { return x }  ->  echo Safe, epi Transparent
        let f = func(
            "f",
            vec!["x"],
            vec![ControlStmt::Return(Some(DataExpr::identifier("x")))],
        );
        let prog = Program {
            statements: vec![TopLevel::Function(f)],
        };
        let env = resolved_effects(&prog);
        assert_eq!(
            env["f"],
            FunctionEffect {
                echo: Echo::Safe,
                epi: Epistemic::Transparent
            }
        );
    }

    #[test]
    fn caller_joins_callee_epistemic() {
        // g(x){ return x }      (epi Transparent)
        // h(){ y = g(0) }       (h calls g -> resolves epi Transparent)
        let g = func(
            "g",
            vec!["x"],
            vec![ControlStmt::Return(Some(DataExpr::identifier("x")))],
        );
        let h = func(
            "h",
            vec![],
            vec![assign(
                "y",
                call("g", vec![DataExpr::Number(Number::Int(0))]),
            )],
        );
        let prog = Program {
            statements: vec![TopLevel::Function(g), TopLevel::Function(h)],
        };
        let env = resolved_effects(&prog);
        assert_eq!(env["g"].epi, Epistemic::Transparent);
        assert_eq!(env["h"].epi, Epistemic::Transparent); // joined from g
    }

    #[test]
    fn caller_joins_callee_echo() {
        // lossy(){ reverse { x += x } }  ->  echo Neutral
        // caller(){ z = lossy() }        ->  resolves echo Neutral via the call
        let lossy = func(
            "lossy",
            vec![],
            vec![ControlStmt::ReverseBlock(ReverseBlock {
                body: vec![ReversibleStmt::AddAssign(
                    "x".to_string(),
                    DataExpr::Identifier("x".to_string()),
                )],
            })],
        );
        let caller = func("caller", vec![], vec![assign("z", call("lossy", vec![]))]);
        let prog = Program {
            statements: vec![TopLevel::Function(lossy), TopLevel::Function(caller)],
        };
        let env = resolved_effects(&prog);
        assert_eq!(env["lossy"].echo, Echo::Neutral);
        assert_eq!(env["caller"].echo, Echo::Neutral); // joined from lossy
    }

    #[test]
    fn recursion_terminates() {
        // f(){ y = f() }  — self-call; the fixpoint must terminate.
        let f = func("f", vec![], vec![assign("y", call("f", vec![]))]);
        let prog = Program {
            statements: vec![TopLevel::Function(f)],
        };
        let env = resolved_effects(&prog);
        assert_eq!(env["f"], FunctionEffect::SAFE);
    }

    #[test]
    fn float_param_reverse_block_resolves_neutral() {
        // fn f(x: float) { reverse { x += y } }
        // Carrier-aware own effect: the float carrier makes the reverse-add
        // Neutral, even though `x += y` has no self-reference.
        let mut f = func(
            "f",
            vec!["x"],
            vec![ControlStmt::ReverseBlock(ReverseBlock {
                body: vec![ReversibleStmt::AddAssign(
                    "x".to_string(),
                    DataExpr::Identifier("y".to_string()),
                )],
            })],
        );
        f.params[0].type_annotation = Some(TypeAnnotation::Basic(BasicType::Float));
        let prog = Program {
            statements: vec![TopLevel::Function(f)],
        };
        let env = resolved_effects(&prog);
        assert_eq!(env["f"].echo, Echo::Neutral);

        // The same body with an int parameter stays Safe.
        let mut g = func(
            "g",
            vec!["x"],
            vec![ControlStmt::ReverseBlock(ReverseBlock {
                body: vec![ReversibleStmt::AddAssign(
                    "x".to_string(),
                    DataExpr::Identifier("y".to_string()),
                )],
            })],
        );
        g.params[0].type_annotation = Some(TypeAnnotation::Basic(BasicType::Int));
        let prog2 = Program {
            statements: vec![TopLevel::Function(g)],
        };
        let env2 = resolved_effects(&prog2);
        assert_eq!(env2["g"].echo, Echo::Safe);
    }
}
