// SPDX-License-Identifier: MPL-2.0
//
// The tokenless reverse gate against what the runtime does.
//
// A tokenless `reverse { x += k }` is admitted only when the checker grades the
// step `Safe`. For that grade to mean anything, the step the runtime then
// executes must keep the value in its carrier, and the carrier must not be an
// approximate one. This test generates small programs that put `x` through a
// token-bound block, an optional reassignment and an optional `reverse tok`,
// then end with a tokenless reverse step, and compares the checker's verdict
// with the carriers the interpreter actually produces.
//
// It exists because the checker tracks carriers statically while tokens are
// reversed dynamically, and review of that tracking found several programs the
// gate admitted over the wrong carrier. Each of those is in the generated set.
//
// Scope: straight-line programs over one variable, with an `if` only inside
// the token-bound block. It says nothing about reassignment through other
// branches or loops (hole H28), nor about which carriers are Safe (H10, H19).

use jtv_core::{number::Value, parser::parse_program, Interpreter, TypeChecker};
use std::mem::discriminant;

fn checks(src: &str) -> bool {
    let program = parse_program(src).unwrap_or_else(|e| panic!("does not parse: {e}\n{src}"));
    TypeChecker::new().check_program(&program).is_ok()
}

/// The value of `x` after running `src`, or `None` if the run fails.
fn run_x(src: &str) -> Option<Value> {
    let program = parse_program(src).ok()?;
    let mut interp = Interpreter::new();
    interp.run(&program).ok()?;
    interp
        .get_variables()
        .into_iter()
        .find(|(name, _)| name == "x")
        .map(|(_, value)| value)
}

fn programs() -> Vec<(String, String)> {
    let inits = ["x = 1", "x = 1/2"];
    let operands = ["1", "1/2", "0.5"];
    let mut blocks: Vec<String> = Vec::new();
    for k in operands {
        blocks.push(format!("  x += {k}\n"));
        blocks.push(format!("  x += 1\n  x += {k}\n"));
        blocks.push(format!("  x -= {k}\n"));
        for v in ["7", "2.5", "2/3"] {
            blocks.push(format!("  if x == 1 {{\n    x = {v}\n  }}\n  x += {k}\n"));
            // The same branch with an `else` that sets an Int: a checker that
            // looks only at the net effect of the `if` misses the change.
            blocks.push(format!(
                "  if x == 1 {{\n    x = {v}\n  }} else {{\n    x = 1\n  }}\n  x += {k}\n"
            ));
            blocks.push(format!(
                "  if x == 2 {{\n    x = 1\n  }} else {{\n    x = {v}\n  }}\n  x += {k}\n"
            ));
        }
    }
    let between = ["", "x = 5\n", "x = 2/3\n", "x = 2.5\n"];
    // `c` is 0 in every program, so the first conditional reversal runs and
    // the second does not. The checker cannot tell them apart.
    let token_action = [
        "",
        "reverse tok\n",
        "if c == 0 {\n  reverse tok\n}\n",
        "if c == 1 {\n  reverse tok\n}\n",
    ];
    let finals = ["1", "1/2"];

    let mut out = Vec::new();
    for init in inits {
        for block in &blocks {
            for mid in between {
                for action in token_action {
                    for k in finals {
                        let prefix = format!(
                            "c = 0\n{init}\nreversible {{\n{block}}} -> tok\n{mid}{action}"
                        );
                        let full = format!("{prefix}reverse {{\n  x += {k}\n}}\n");
                        out.push((prefix, full));
                    }
                }
            }
        }
    }
    out
}

#[test]
fn an_admitted_tokenless_step_keeps_an_exact_carrier_at_run_time() {
    let mut admitted = 0;
    let mut rejected = 0;
    let mut violations = Vec::new();

    for (prefix, full) in programs() {
        if !checks(&full) {
            rejected += 1;
            continue;
        }
        // Both runs must succeed for the comparison to mean anything.
        let (Some(before), Some(after)) = (run_x(&prefix), run_x(&full)) else {
            continue;
        };
        admitted += 1;
        let promoted = discriminant(&before) != discriminant(&after);
        let approximate = matches!(before, Value::Float(_) | Value::Complex(_));
        if promoted || approximate {
            violations.push(format!(
                "admitted, but x went from {before:?} to {after:?}:\n{full}"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "{} of {admitted} admitted programs broke the gate's promise:\n{}",
        violations.len(),
        violations.join("\n")
    );
    // The set must exercise both verdicts, or the test proves nothing.
    assert!(admitted > 50, "only {admitted} programs were admitted");
    assert!(rejected > 50, "only {rejected} programs were rejected");
}
