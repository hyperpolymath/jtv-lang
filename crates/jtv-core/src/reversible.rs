// Reversible computing for Julia the Viper (v2 feature)
// Implements automatic reverse execution for reverse blocks

use crate::ast::*;
use crate::error::{JtvError, Result};
use crate::number::Value;
use std::collections::HashMap;

/// A recorded operation that can be reversed
#[derive(Debug, Clone)]
pub enum RecordedOp {
    /// x += value (reverse: x -= value)
    AddAssign { target: String, value: Value },
    /// x -= value (reverse: x += value)
    SubAssign { target: String, value: Value },
    /// Conditional branch (reverse requires same condition)
    If {
        condition_was_true: bool,
        then_ops: Vec<RecordedOp>,
        else_ops: Vec<RecordedOp>,
    },
    /// Residue/token (Bennett) record: the value of `target` that a
    /// self-referential (Neutral) step overwrote, saved *before* the step. The
    /// naive `-value` inverse is wrong for self-reference, so this is reversed
    /// by restoring `old_value`. Runtime counterpart of `execBackwardWithToken`
    /// in `jtv_proofs/JtvTheorems.lean` (the v2 "(c)" Neutral bridge).
    Snapshot { target: String, old_value: Value },
}

impl RecordedOp {
    /// Create the inverse operation
    pub fn inverse(&self) -> RecordedOp {
        match self {
            RecordedOp::AddAssign { target, value } => RecordedOp::SubAssign {
                target: target.clone(),
                value: value.clone(),
            },
            RecordedOp::SubAssign { target, value } => RecordedOp::AddAssign {
                target: target.clone(),
                value: value.clone(),
            },
            RecordedOp::If {
                condition_was_true,
                then_ops,
                else_ops,
            } => RecordedOp::If {
                condition_was_true: *condition_was_true,
                then_ops: then_ops.iter().rev().map(|op| op.inverse()).collect(),
                else_ops: else_ops.iter().rev().map(|op| op.inverse()).collect(),
            },
            // A snapshot's inverse is the snapshot itself: applying it restores
            // the saved residue (`old_value`), which is exactly the undo of the
            // step that overwrote `target`.
            RecordedOp::Snapshot { target, old_value } => RecordedOp::Snapshot {
                target: target.clone(),
                old_value: old_value.clone(),
            },
        }
    }
}

/// Execution trace for a reverse block
#[derive(Debug, Clone)]
pub struct ReverseTrace {
    /// Operations recorded during forward execution
    operations: Vec<RecordedOp>,
}

impl ReverseTrace {
    pub fn new() -> Self {
        ReverseTrace { operations: vec![] }
    }

    /// Record an operation
    pub fn record(&mut self, op: RecordedOp) {
        self.operations.push(op);
    }

    /// Get operations in reverse order with inverses
    pub fn reverse_operations(&self) -> Vec<RecordedOp> {
        self.operations
            .iter()
            .rev()
            .map(|op| op.inverse())
            .collect()
    }
}

impl Default for ReverseTrace {
    fn default() -> Self {
        Self::new()
    }
}

/// Reversible interpreter that records operations for reversal
pub struct ReversibleInterpreter {
    variables: HashMap<String, Value>,
    trace: ReverseTrace,
}

impl ReversibleInterpreter {
    pub fn new() -> Self {
        ReversibleInterpreter {
            variables: HashMap::new(),
            trace: ReverseTrace::new(),
        }
    }

    /// Create from existing state
    pub fn with_state(variables: HashMap<String, Value>) -> Self {
        ReversibleInterpreter {
            variables,
            trace: ReverseTrace::new(),
        }
    }

    /// Execute a reverse block forward, recording operations
    pub fn execute_forward(&mut self, block: &ReverseBlock) -> Result<()> {
        for stmt in &block.body {
            self.execute_reversible_stmt(stmt)?;
        }
        Ok(())
    }

    /// Consume and return the operations recorded so far during a forward pass.
    ///
    /// Used by the `reversible { } -> tok` interpreter path to move the log out
    /// of this interpreter and into the host's token store. Leaves the trace
    /// empty afterwards (`std::mem::take`), so a subsequent forward pass starts
    /// from a clean log. The ops are returned in forward declaration order; the
    /// inverse/replay path (`apply_inverse_ops`) handles reversal.
    pub fn take_recorded_ops(&mut self) -> Vec<RecordedOp> {
        std::mem::take(&mut self.trace).operations
    }

    /// Apply the inverse of a previously-recorded operation log to the current
    /// state, in reverse declaration order.
    ///
    /// This is the replay counterpart to `take_recorded_ops`: given the ops a
    /// `reversible { }` block recorded (in forward order), undoing them means
    /// applying each op's `inverse()` from last to first. Used by the
    /// `reverse tok` interpreter path.
    pub fn apply_inverse_ops(&mut self, ops: &[RecordedOp]) -> Result<()> {
        for op in ops.iter().rev() {
            self.apply_operation(&op.inverse())?;
        }
        Ok(())
    }

    /// Execute the reverse of recorded operations
    pub fn execute_reverse(&mut self) -> Result<()> {
        let reverse_ops = self.trace.reverse_operations();
        for op in reverse_ops {
            self.apply_operation(&op)?;
        }
        // Clear the trace after reversal
        self.trace = ReverseTrace::new();
        Ok(())
    }

    /// Execute forward then reverse (should return to original state).
    /// This is the CNO (Certified Null Operation) pattern — the net effect
    /// on state is identity.  Used when you want to prove reversibility.
    pub fn execute_and_reverse(&mut self, block: &ReverseBlock) -> Result<()> {
        self.execute_forward(block)?;
        self.execute_reverse()
    }

    /// Apply the INVERSE of each operation in `block`, in reverse declaration order.
    ///
    /// This implements JtV v2's core claim: `reverse { x += v }` IS subtraction.
    /// The inverse of `AddAssign` is `SubAssign`, and vice versa.  Operations are
    /// applied in reverse order so that multi-step blocks invert correctly.
    ///
    /// Expressions are evaluated against the state as it evolves through the
    /// inversion (same semantics as the forward pass, but in reverse).
    pub fn execute_inverse(&mut self, block: &ReverseBlock) -> Result<()> {
        for stmt in block.body.iter().rev() {
            self.execute_inverted_stmt(stmt)?;
        }
        Ok(())
    }

    fn execute_inverted_stmt(&mut self, stmt: &ReversibleStmt) -> Result<()> {
        match stmt {
            ReversibleStmt::AddAssign(target, expr) => {
                // Inverse of x += v is x -= v
                let value = self.eval_data_expr(expr)?;
                let current = self.get_variable(target)?;
                let neg_value = value.negate()?;
                let result = current.add(&neg_value)?;
                self.variables.insert(target.clone(), result);
                Ok(())
            }
            ReversibleStmt::SubAssign(target, expr) => {
                // Inverse of x -= v is x += v
                let value = self.eval_data_expr(expr)?;
                let current = self.get_variable(target)?;
                let result = current.add(&value)?;
                self.variables.insert(target.clone(), result);
                Ok(())
            }
            ReversibleStmt::If(if_stmt) => {
                // Re-evaluate the condition on current state (Janus requirement:
                // the condition must hold in the same direction when reversing).
                let cond = self.eval_control_expr(&if_stmt.condition)?;
                if cond.is_truthy() {
                    for ctrl_stmt in if_stmt.then_branch.iter().rev() {
                        if let crate::ast::ControlStmt::ReverseBlock(inner) = ctrl_stmt {
                            self.execute_inverse(inner)?;
                        }
                        // Non-reverse stmts in reverse-block if branches are
                        // rejected by check_reversibility before we get here.
                    }
                } else if let Some(else_branch) = &if_stmt.else_branch {
                    for ctrl_stmt in else_branch.iter().rev() {
                        if let crate::ast::ControlStmt::ReverseBlock(inner) = ctrl_stmt {
                            self.execute_inverse(inner)?;
                        }
                    }
                }
                Ok(())
            }
        }
    }

    fn execute_reversible_stmt(&mut self, stmt: &ReversibleStmt) -> Result<()> {
        match stmt {
            ReversibleStmt::AddAssign(target, expr) => {
                let value = self.eval_data_expr(expr)?;
                let current = self.get_variable(target)?;
                let new_value = current.add(&value)?;

                // Record the operation. Self-reference (`x += x`) is the Neutral
                // tier: the naive `-value` inverse is wrong, so snapshot the
                // overwritten value (the residue/token) and invert by restoring
                // it (Bennett). Independent targets are Safe: record the added
                // value and invert by subtraction.
                if expr_contains_var(expr, target) {
                    self.trace.record(RecordedOp::Snapshot {
                        target: target.clone(),
                        old_value: current,
                    });
                } else {
                    self.trace.record(RecordedOp::AddAssign {
                        target: target.clone(),
                        value: value.clone(),
                    });
                }

                self.variables.insert(target.clone(), new_value);
                Ok(())
            }
            ReversibleStmt::SubAssign(target, expr) => {
                let value = self.eval_data_expr(expr)?;
                let current = self.get_variable(target)?;
                let neg_value = value.negate()?;
                let new_value = current.add(&neg_value)?;

                // See AddAssign: self-reference snapshots the residue (Neutral);
                // independent targets record the value and invert by addition.
                if expr_contains_var(expr, target) {
                    self.trace.record(RecordedOp::Snapshot {
                        target: target.clone(),
                        old_value: current,
                    });
                } else {
                    self.trace.record(RecordedOp::SubAssign {
                        target: target.clone(),
                        value: value.clone(),
                    });
                }

                self.variables.insert(target.clone(), new_value);
                Ok(())
            }
            ReversibleStmt::If(if_stmt) => {
                let condition = self.eval_control_expr(&if_stmt.condition)?;
                let condition_true = condition.is_truthy();

                // Create sub-interpreter to track nested ops
                let mut then_trace = ReverseTrace::new();
                let mut else_trace = ReverseTrace::new();

                if condition_true {
                    // Execute then branch and record
                    let old_trace = std::mem::take(&mut self.trace);
                    for stmt in &if_stmt.then_branch {
                        self.execute_control_stmt_reversible(stmt)?;
                    }
                    then_trace = std::mem::replace(&mut self.trace, old_trace);
                } else if let Some(else_branch) = &if_stmt.else_branch {
                    // Execute else branch and record
                    let old_trace = std::mem::take(&mut self.trace);
                    for stmt in else_branch {
                        self.execute_control_stmt_reversible(stmt)?;
                    }
                    else_trace = std::mem::replace(&mut self.trace, old_trace);
                }

                // Record the if as a single operation
                self.trace.record(RecordedOp::If {
                    condition_was_true: condition_true,
                    then_ops: then_trace.operations,
                    else_ops: else_trace.operations,
                });

                Ok(())
            }
        }
    }

    fn execute_control_stmt_reversible(&mut self, stmt: &ControlStmt) -> Result<()> {
        // Only assignments are allowed in reversible context
        if let ControlStmt::Assignment(assign) = stmt {
            let value = match &assign.value {
                Expr::Data(expr) => self.eval_data_expr(expr)?,
                Expr::Control(_) => {
                    return Err(JtvError::RuntimeError(
                        "Control expressions not allowed in reverse blocks".to_string(),
                    ))
                }
            };
            self.variables.insert(assign.target.clone(), value);
            Ok(())
        } else {
            Err(JtvError::RuntimeError(
                "Only assignments allowed in reversible if branches".to_string(),
            ))
        }
    }

    fn apply_operation(&mut self, op: &RecordedOp) -> Result<()> {
        match op {
            RecordedOp::AddAssign { target, value } => {
                let current = self.get_variable(target)?;
                let new_value = current.add(value)?;
                self.variables.insert(target.clone(), new_value);
                Ok(())
            }
            RecordedOp::SubAssign { target, value } => {
                let current = self.get_variable(target)?;
                let neg_value = value.negate()?;
                let new_value = current.add(&neg_value)?;
                self.variables.insert(target.clone(), new_value);
                Ok(())
            }
            RecordedOp::If {
                condition_was_true,
                then_ops,
                else_ops,
            } => {
                // Apply the appropriate branch's operations
                let ops = if *condition_was_true {
                    then_ops
                } else {
                    else_ops
                };
                for nested_op in ops {
                    self.apply_operation(nested_op)?;
                }
                Ok(())
            }
            // Restore the saved residue — the Bennett undo of a self-referential
            // (Neutral) overwrite.
            RecordedOp::Snapshot { target, old_value } => {
                self.variables.insert(target.clone(), old_value.clone());
                Ok(())
            }
        }
    }

    fn eval_control_expr(&self, expr: &ControlExpr) -> Result<Value> {
        match expr {
            ControlExpr::Data(data) => self.eval_data_expr(data),
            ControlExpr::Comparison(left, op, right) => {
                let left_val = self.eval_data_expr(left)?;
                let right_val = self.eval_data_expr(right)?;
                let result = match op {
                    Comparator::Eq => left_val.eq(&right_val)?,
                    Comparator::Ne => left_val.ne(&right_val)?,
                    Comparator::Lt => left_val.lt(&right_val)?,
                    Comparator::Le => left_val.le(&right_val)?,
                    Comparator::Gt => left_val.gt(&right_val)?,
                    Comparator::Ge => left_val.ge(&right_val)?,
                };
                Ok(Value::Bool(result))
            }
            ControlExpr::Logical(left, op, right) => {
                let left_val = self.eval_control_expr(left)?;
                match op {
                    LogicalOp::And => {
                        if !left_val.is_truthy() {
                            Ok(Value::Bool(false))
                        } else {
                            let right_val = self.eval_control_expr(right)?;
                            Ok(Value::Bool(right_val.is_truthy()))
                        }
                    }
                    LogicalOp::Or => {
                        if left_val.is_truthy() {
                            Ok(Value::Bool(true))
                        } else {
                            let right_val = self.eval_control_expr(right)?;
                            Ok(Value::Bool(right_val.is_truthy()))
                        }
                    }
                }
            }
            ControlExpr::Not(inner) => {
                let val = self.eval_control_expr(inner)?;
                Ok(Value::Bool(!val.is_truthy()))
            }
        }
    }

    fn eval_data_expr(&self, expr: &DataExpr) -> Result<Value> {
        match expr {
            DataExpr::Number(num) => Value::from_number(num),
            DataExpr::StringLit(s) => Ok(Value::String(s.clone())),
            DataExpr::Identifier(name) => self.get_variable(name),
            DataExpr::Add(left, right) => {
                let left_val = self.eval_data_expr(left)?;
                let right_val = self.eval_data_expr(right)?;
                left_val.add(&right_val)
            }
            DataExpr::Negate(inner) => {
                let value = self.eval_data_expr(inner)?;
                value.negate()
            }
            DataExpr::FunctionCall(_) => Err(JtvError::RuntimeError(
                "Function calls not supported in reversible context".to_string(),
            )),
            DataExpr::List(_) | DataExpr::Tuple(_) => Err(JtvError::RuntimeError(
                "Collections not supported in reversible context".to_string(),
            )),
        }
    }

    fn get_variable(&self, name: &str) -> Result<Value> {
        self.variables
            .get(name)
            .cloned()
            .ok_or_else(|| JtvError::UndefinedVariable(name.to_string()))
    }

    /// Get current state
    pub fn get_state(&self) -> &HashMap<String, Value> {
        &self.variables
    }

    /// Get a specific variable
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Set a variable
    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }
}

impl Default for ReversibleInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify that a reverse block is properly reversible
/// (i.e., no variable appears on both sides of an assignment involving itself)
pub fn check_reversibility(block: &ReverseBlock) -> Result<()> {
    for stmt in &block.body {
        check_reversible_stmt(stmt)?;
    }
    Ok(())
}

fn check_reversible_stmt(stmt: &ReversibleStmt) -> Result<()> {
    match stmt {
        ReversibleStmt::AddAssign(target, expr) | ReversibleStmt::SubAssign(target, expr) => {
            // Check that target doesn't appear in expr (would break reversibility)
            if expr_contains_var(expr, target) {
                return Err(JtvError::RuntimeError(format!(
                    "Variable '{}' cannot appear in its own reversible assignment (breaks reversibility)",
                    target
                )));
            }
            Ok(())
        }
        ReversibleStmt::If(if_stmt) => {
            // Recursively check branches
            for stmt in &if_stmt.then_branch {
                if let ControlStmt::ReverseBlock(block) = stmt {
                    check_reversibility(block)?;
                }
            }
            if let Some(else_branch) = &if_stmt.else_branch {
                for stmt in else_branch {
                    if let ControlStmt::ReverseBlock(block) = stmt {
                        check_reversibility(block)?;
                    }
                }
            }
            Ok(())
        }
    }
}

fn expr_contains_var(expr: &DataExpr, var: &str) -> bool {
    match expr {
        DataExpr::Number(_) => false,
        DataExpr::StringLit(_) => false,
        DataExpr::Identifier(name) => name == var,
        DataExpr::Add(left, right) => expr_contains_var(left, var) || expr_contains_var(right, var),
        DataExpr::Negate(inner) => expr_contains_var(inner, var),
        DataExpr::FunctionCall(call) => call.args.iter().any(|arg| expr_contains_var(arg, var)),
        DataExpr::List(elems) => elems.iter().any(|e| expr_contains_var(e, var)),
        DataExpr::Tuple(elems) => elems.iter().any(|e| expr_contains_var(e, var)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_execution() {
        let mut interp = ReversibleInterpreter::new();
        interp.set("x".to_string(), Value::Int(10));

        let block = ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::Number(Number::Int(5)),
            )],
        };

        interp.execute_forward(&block).unwrap();
        assert_eq!(interp.get("x"), Some(&Value::Int(15)));
    }

    #[test]
    fn test_reverse_execution() {
        let mut interp = ReversibleInterpreter::new();
        interp.set("x".to_string(), Value::Int(10));

        let block = ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::Number(Number::Int(5)),
            )],
        };

        interp.execute_forward(&block).unwrap();
        assert_eq!(interp.get("x"), Some(&Value::Int(15)));

        interp.execute_reverse().unwrap();
        assert_eq!(interp.get("x"), Some(&Value::Int(10))); // Back to original!
    }

    #[test]
    fn test_multiple_operations_reverse() {
        let mut interp = ReversibleInterpreter::new();
        interp.set("x".to_string(), Value::Int(10));
        interp.set("y".to_string(), Value::Int(20));

        let block = ReverseBlock {
            body: vec![
                ReversibleStmt::AddAssign("x".to_string(), DataExpr::Number(Number::Int(5))),
                ReversibleStmt::SubAssign("y".to_string(), DataExpr::Number(Number::Int(3))),
                ReversibleStmt::AddAssign("x".to_string(), DataExpr::Identifier("y".to_string())),
            ],
        };

        let original_x = interp.get("x").cloned();
        let original_y = interp.get("y").cloned();

        interp.execute_forward(&block).unwrap();
        // x = 10 + 5 = 15, y = 20 - 3 = 17, x = 15 + 17 = 32
        assert_eq!(interp.get("x"), Some(&Value::Int(32)));
        assert_eq!(interp.get("y"), Some(&Value::Int(17)));

        interp.execute_reverse().unwrap();
        // Should be back to original
        assert_eq!(interp.get("x"), original_x.as_ref());
        assert_eq!(interp.get("y"), original_y.as_ref());
    }

    #[test]
    fn test_reversibility_check_fails() {
        // x += x is not reversible because we can't recover original x
        let block = ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::Identifier("x".to_string()),
            )],
        };

        assert!(check_reversibility(&block).is_err());
    }

    #[test]
    fn test_reversibility_check_passes() {
        // x += y is reversible (y is independent)
        let block = ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::Identifier("y".to_string()),
            )],
        };

        assert!(check_reversibility(&block).is_ok());
    }

    #[test]
    fn test_execute_and_reverse_identity() {
        let mut interp = ReversibleInterpreter::new();
        interp.set("a".to_string(), Value::Int(100));
        interp.set("b".to_string(), Value::Int(50));

        let original_state = interp.get_state().clone();

        let block = ReverseBlock {
            body: vec![
                ReversibleStmt::AddAssign("a".to_string(), DataExpr::Number(Number::Int(25))),
                ReversibleStmt::SubAssign("b".to_string(), DataExpr::Number(Number::Int(10))),
            ],
        };

        interp.execute_and_reverse(&block).unwrap();

        // State should be identical to original
        assert_eq!(interp.get_state(), &original_state);
    }

    #[test]
    fn test_neutral_self_reference_recovered_via_residue() {
        // The v2 "(c)" Neutral bridge at runtime: `x += x` is lossy under the
        // naive `-` inverse, but the recorded residue (a snapshot of the old x)
        // restores it exactly — forward then reverse = identity.
        let mut interp = ReversibleInterpreter::new();
        interp.set("x".to_string(), Value::Int(5));

        let block = ReverseBlock {
            body: vec![ReversibleStmt::AddAssign(
                "x".to_string(),
                DataExpr::Identifier("x".to_string()),
            )],
        };

        // Forward `x += x`  ->  x = 10, snapshotting old x = 5.
        interp.execute_forward(&block).unwrap();
        assert_eq!(interp.get("x"), Some(&Value::Int(10)));

        // Reverse via the recorded residue restores the original x = 5
        // (the naive `x -= x` inverse would wrongly yield 0).
        interp.execute_reverse().unwrap();
        assert_eq!(interp.get("x"), Some(&Value::Int(5)));
    }
}
