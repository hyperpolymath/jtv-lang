// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// JtV - Pretty-Printer
// Converts AST nodes back to well-formatted JtV source code.
// Unlike the Formatter (which parses source then reformats), the
// pretty-printer works directly on AST nodes and supports individual
// node rendering for diagnostics and REPL output.

use crate::ast::*;
use std::fmt;

// ===== DISPLAY IMPLEMENTATIONS =====

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printer = PrettyPrinter::new();
        write!(f, "{}", printer.print_program(self))
    }
}

impl fmt::Display for TopLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printer = PrettyPrinter::new();
        write!(f, "{}", printer.print_top_level(self, 0))
    }
}

impl fmt::Display for ControlStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printer = PrettyPrinter::new();
        write!(f, "{}", printer.print_control_stmt(self, 0))
    }
}

impl fmt::Display for DataExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printer = PrettyPrinter::new();
        write!(f, "{}", printer.print_data_expr(self))
    }
}

impl fmt::Display for ControlExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printer = PrettyPrinter::new();
        write!(f, "{}", printer.print_control_expr(self))
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printer = PrettyPrinter::new();
        write!(f, "{}", printer.print_number(self))
    }
}

impl fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let printer = PrettyPrinter::new();
        write!(f, "{}", printer.print_type_annotation(self))
    }
}

impl fmt::Display for Purity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Purity::Pure => write!(f, "@pure"),
            Purity::Total => write!(f, "@total"),
            Purity::Impure => Ok(()),
        }
    }
}

impl fmt::Display for Comparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Comparator::Eq => write!(f, "=="),
            Comparator::Ne => write!(f, "!="),
            Comparator::Lt => write!(f, "<"),
            Comparator::Le => write!(f, "<="),
            Comparator::Gt => write!(f, ">"),
            Comparator::Ge => write!(f, ">="),
        }
    }
}

impl fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOp::And => write!(f, "&&"),
            LogicalOp::Or => write!(f, "||"),
        }
    }
}

// ===== PRETTY-PRINTER ENGINE =====

/// Pretty-printer for JtV AST nodes.
///
/// Renders AST nodes as well-formatted JtV source code with
/// consistent indentation, spacing, and line breaks.
pub struct PrettyPrinter {
    /// Number of spaces per indentation level.
    indent_width: usize,
}

impl PrettyPrinter {
    /// Create a pretty-printer with the default indent width (4 spaces).
    pub fn new() -> Self {
        Self { indent_width: 4 }
    }

    /// Create a pretty-printer with a custom indent width.
    pub fn with_indent(indent_width: usize) -> Self {
        Self { indent_width }
    }

    /// Render a complete program.
    pub fn print_program(&self, program: &Program) -> String {
        let mut out = String::new();
        for (i, stmt) in program.statements.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&self.print_top_level(stmt, 0));
            out.push('\n');
        }
        out
    }

    /// Render a top-level item at the given indentation depth.
    pub fn print_top_level(&self, item: &TopLevel, depth: usize) -> String {
        match item {
            TopLevel::Module(module) => self.print_module(module, depth),
            TopLevel::Import(import) => self.print_import(import, depth),
            TopLevel::Function(func) => self.print_function(func, depth),
            TopLevel::Control(stmt) => self.print_control_stmt(stmt, depth),
            TopLevel::ExternCoproc(block) => {
                let indent = self.indent(depth);
                let status = match &block.resolved {
                    Some(r) if r.live => format!(" /* live, family: {} */", r.family),
                    Some(_) => " /* dead */".into(),
                    None => " /* unresolved */".into(),
                };
                format!(
                    "{}extern coproc {}{} {{ /* {} item(s) */ }}\n",
                    indent,
                    block.gate_name,
                    status,
                    block.items.len()
                )
            }
        }
    }

    /// Render a module declaration.
    fn print_module(&self, module: &ModuleDecl, depth: usize) -> String {
        let indent = self.indent(depth);
        let mut out = format!("{}module {} {{\n", indent, module.name);

        for (i, item) in module.body.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&self.print_top_level(item, depth + 1));
            out.push('\n');
        }

        out.push_str(&format!("{}}}", indent));
        out
    }

    /// Render an import statement.
    fn print_import(&self, import: &ImportStmt, depth: usize) -> String {
        let indent = self.indent(depth);
        let path = import.path.join(".");
        match &import.alias {
            Some(alias) => format!("{}import {} as {}", indent, path, alias),
            None => format!("{}import {}", indent, path),
        }
    }

    /// Render a function declaration.
    fn print_function(&self, func: &FunctionDecl, depth: usize) -> String {
        let indent = self.indent(depth);
        let mut out = String::new();

        // Purity marker
        out.push_str(&indent);
        match func.purity {
            Purity::Pure => out.push_str("@pure "),
            Purity::Total => out.push_str("@total "),
            Purity::Impure => {}
        }

        // Echo grade annotation
        if let Some(echo) = func.echo_annotation {
            out.push_str(&format!(
                "@echo({}) ",
                match echo {
                    Echo::Safe => "Safe",
                    Echo::Neutral => "Neutral",
                    Echo::Breaking => "Breaking",
                }
            ));
        }

        // Signature
        out.push_str("fn ");
        out.push_str(&func.name);
        out.push('(');

        let params: Vec<String> = func.params.iter().map(|p| self.print_param(p)).collect();
        out.push_str(&params.join(", "));
        out.push(')');

        if let Some(ret) = &func.return_type {
            out.push_str(": ");
            out.push_str(&self.print_type_annotation(ret));
        }

        out.push_str(" {\n");

        // Body
        for stmt in &func.body {
            out.push_str(&self.print_control_stmt(stmt, depth + 1));
            out.push('\n');
        }

        out.push_str(&format!("{}}}", indent));
        out
    }

    /// Render a parameter.
    fn print_param(&self, param: &Param) -> String {
        match &param.type_annotation {
            Some(ty) => format!("{}: {}", param.name, self.print_type_annotation(ty)),
            None => param.name.clone(),
        }
    }

    /// Render a control statement.
    pub fn print_control_stmt(&self, stmt: &ControlStmt, depth: usize) -> String {
        let indent = self.indent(depth);

        match stmt {
            ControlStmt::Assignment(assign) => {
                format!(
                    "{}{} = {}",
                    indent,
                    assign.target,
                    self.print_expr(&assign.value)
                )
            }
            ControlStmt::If(if_stmt) => self.print_if(if_stmt, depth),
            ControlStmt::While(while_stmt) => {
                let mut out = format!(
                    "{}while {} {{\n",
                    indent,
                    self.print_control_expr(&while_stmt.condition)
                );
                for s in &while_stmt.body {
                    out.push_str(&self.print_control_stmt(s, depth + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{}}}", indent));
                out
            }
            ControlStmt::For(for_stmt) => {
                let range = self.print_range(&for_stmt.range);
                let mut out = format!("{}for {} in {} {{\n", indent, for_stmt.variable, range);
                for s in &for_stmt.body {
                    out.push_str(&self.print_control_stmt(s, depth + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{}}}", indent));
                out
            }
            ControlStmt::Return(expr) => match expr {
                Some(e) => format!("{}return {}", indent, self.print_data_expr(e)),
                None => format!("{}return", indent),
            },
            ControlStmt::Print(exprs) => {
                let args: Vec<String> = exprs.iter().map(|e| self.print_data_expr(e)).collect();
                format!("{}print({})", indent, args.join(", "))
            }
            ControlStmt::ReverseBlock(block) => {
                let mut out = format!("{}reverse {{\n", indent);
                for s in &block.body {
                    out.push_str(&self.print_reversible_stmt(s, depth + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{}}}", indent));
                out
            }
            ControlStmt::ReversibleBlock(rb) => {
                let mut out = format!("{}reversible {{\n", indent);
                for s in &rb.body {
                    out.push_str(&self.print_reversible_stmt(s, depth + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{}}}", indent));
                if let Some(tok) = &rb.token_binding {
                    out.push_str(&format!(" -> {}", tok));
                }
                out
            }
            ControlStmt::ReverseToken(tok) => format!("{}reverse {}", indent, tok),
            ControlStmt::AbandonToken(tok) => format!("{}abandon {}", indent, tok),
            ControlStmt::Block(stmts) => {
                let mut out = format!("{}{{\n", indent);
                for s in stmts {
                    out.push_str(&self.print_control_stmt(s, depth + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{}}}", indent));
                out
            }
        }
    }

    /// Render an if statement.
    fn print_if(&self, if_stmt: &IfStmt, depth: usize) -> String {
        let indent = self.indent(depth);
        let mut out = format!(
            "{}if {} {{\n",
            indent,
            self.print_control_expr(&if_stmt.condition)
        );

        for s in &if_stmt.then_branch {
            out.push_str(&self.print_control_stmt(s, depth + 1));
            out.push('\n');
        }

        match &if_stmt.else_branch {
            Some(else_stmts) => {
                out.push_str(&format!("{}}} else {{\n", indent));
                for s in else_stmts {
                    out.push_str(&self.print_control_stmt(s, depth + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{}}}", indent));
            }
            None => {
                out.push_str(&format!("{}}}", indent));
            }
        }

        out
    }

    /// Render a reversible statement.
    fn print_reversible_stmt(&self, stmt: &ReversibleStmt, depth: usize) -> String {
        let indent = self.indent(depth);
        match stmt {
            ReversibleStmt::AddAssign(target, expr) => {
                format!("{}{} += {}", indent, target, self.print_data_expr(expr))
            }
            ReversibleStmt::SubAssign(target, expr) => {
                format!("{}{} -= {}", indent, target, self.print_data_expr(expr))
            }
            ReversibleStmt::If(if_stmt) => self.print_if(if_stmt, depth),
        }
    }

    /// Render an expression (data or control).
    pub fn print_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Data(d) => self.print_data_expr(d),
            Expr::Control(c) => self.print_control_expr(c),
        }
    }

    /// Render a data expression.
    pub fn print_data_expr(&self, expr: &DataExpr) -> String {
        match expr {
            DataExpr::Number(n) => self.print_number(n),
            DataExpr::StringLit(s) => format!("\"{}\"", s),
            DataExpr::Identifier(name) => name.clone(),
            DataExpr::Add(left, right) => {
                format!(
                    "{} + {}",
                    self.print_data_expr(left),
                    self.print_data_expr(right)
                )
            }
            DataExpr::Negate(inner) => {
                format!("-{}", self.print_data_expr(inner))
            }
            DataExpr::FunctionCall(call) => {
                let name = match &call.module {
                    Some(path) => format!("{}.{}", path.join("."), call.name),
                    None => call.name.clone(),
                };
                let args: Vec<String> = call.args.iter().map(|a| self.print_data_expr(a)).collect();
                format!("{}({})", name, args.join(", "))
            }
            DataExpr::List(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| self.print_data_expr(e)).collect();
                format!("[{}]", elems.join(", "))
            }
            DataExpr::Tuple(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| self.print_data_expr(e)).collect();
                format!("({})", elems.join(", "))
            }
        }
    }

    /// Render a control expression.
    pub fn print_control_expr(&self, expr: &ControlExpr) -> String {
        match expr {
            ControlExpr::Data(d) => self.print_data_expr(d),
            ControlExpr::Comparison(left, op, right) => {
                format!(
                    "{} {} {}",
                    self.print_data_expr(left),
                    op,
                    self.print_data_expr(right)
                )
            }
            ControlExpr::Logical(left, op, right) => {
                format!(
                    "{} {} {}",
                    self.print_control_expr(left),
                    op,
                    self.print_control_expr(right)
                )
            }
            ControlExpr::Not(inner) => {
                format!("!{}", self.print_control_expr(inner))
            }
        }
    }

    /// Render a number literal.
    pub fn print_number(&self, num: &Number) -> String {
        match num {
            Number::Int(n) => n.to_string(),
            Number::Float(f) => {
                let s = f.to_string();
                if s.contains('.') {
                    s
                } else {
                    format!("{}.0", s)
                }
            }
            Number::Rational(num, den) => format!("{}/{}", num, den),
            Number::Complex(re, im) => {
                if *re != 0.0 {
                    if *im >= 0.0 {
                        format!("{}+{}i", re, im)
                    } else {
                        format!("{}{}i", re, im)
                    }
                } else {
                    format!("{}i", im)
                }
            }
            Number::Hex(s) => s.clone(),
            Number::Binary(s) => s.clone(),
            Number::Symbolic(s) => format!("#{}", s),
        }
    }

    /// Render a type annotation.
    pub fn print_type_annotation(&self, ty: &TypeAnnotation) -> String {
        match ty {
            TypeAnnotation::Basic(basic) => match basic {
                BasicType::Int => "Int".to_string(),
                BasicType::Float => "Float".to_string(),
                BasicType::Rational => "Rational".to_string(),
                BasicType::Complex => "Complex".to_string(),
                BasicType::Hex => "Hex".to_string(),
                BasicType::Binary => "Binary".to_string(),
                BasicType::Symbolic => "Symbolic".to_string(),
                BasicType::Bool => "Bool".to_string(),
                BasicType::String => "String".to_string(),
            },
            TypeAnnotation::List(inner) => {
                format!("List<{}>", self.print_type_annotation(inner))
            }
            TypeAnnotation::Tuple(types) => {
                let parts: Vec<String> = types
                    .iter()
                    .map(|t| self.print_type_annotation(t))
                    .collect();
                format!("({})", parts.join(", "))
            }
            TypeAnnotation::Function(params, ret) => {
                let parts: Vec<String> = params
                    .iter()
                    .map(|t| self.print_type_annotation(t))
                    .collect();
                format!(
                    "Fn({}) -> {}",
                    parts.join(", "),
                    self.print_type_annotation(ret)
                )
            }
        }
    }

    /// Render a range expression.
    fn print_range(&self, range: &RangeExpr) -> String {
        let mut out = format!(
            "{}..{}",
            self.print_data_expr(&range.start),
            self.print_data_expr(&range.end)
        );
        if let Some(step) = &range.step {
            out.push_str(&format!("..{}", self.print_data_expr(step)));
        }
        out
    }

    /// Produce an indentation string for the given depth.
    fn indent(&self, depth: usize) -> String {
        " ".repeat(depth * self.indent_width)
    }
}

impl Default for PrettyPrinter {
    fn default() -> Self {
        Self::new()
    }
}

// ===== CONVENIENCE FUNCTIONS =====

/// Pretty-print a program AST back to source code.
pub fn pretty_print(program: &Program) -> String {
    let printer = PrettyPrinter::new();
    printer.print_program(program)
}

/// Pretty-print a single data expression.
pub fn pretty_print_data_expr(expr: &DataExpr) -> String {
    let printer = PrettyPrinter::new();
    printer.print_data_expr(expr)
}

/// Pretty-print a single control statement.
pub fn pretty_print_stmt(stmt: &ControlStmt) -> String {
    let printer = PrettyPrinter::new();
    printer.print_control_stmt(stmt, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_program;

    /// Round-trip test: parse -> pretty-print -> parse again.
    /// The two ASTs should be identical.
    fn round_trip(code: &str) {
        let ast1 = parse_program(code).expect("First parse should succeed");
        let printed = pretty_print(&ast1);
        let ast2 = parse_program(&printed).unwrap_or_else(|e| {
            panic!("Second parse should succeed ({e:?}). Pretty-printed:\n{printed}")
        });
        assert_eq!(
            ast1, ast2,
            "Round-trip mismatch. Pretty-printed:\n{}",
            printed
        );
    }

    #[test]
    fn test_round_trip_assignment() {
        round_trip("x = 5 + 3");
    }

    #[test]
    fn test_round_trip_function() {
        round_trip("fn add(a: Int, b: Int): Int { return a + b }");
    }

    #[test]
    fn test_round_trip_echo_annotation() {
        // The @echo(...) grade must survive parse -> print -> parse.
        round_trip("@echo(Neutral) fn double(x: Int): Int { return x + x }");
        round_trip("@pure @echo(Breaking) fn id(x: Int): Int { return x }");
    }

    #[test]
    fn test_round_trip_pure_function() {
        round_trip("@pure fn double(x: Int): Int { return x + x }");
    }

    #[test]
    fn test_round_trip_total_function() {
        round_trip("@total fn identity(x: Int): Int { return x }");
    }

    #[test]
    fn test_round_trip_if_else() {
        round_trip("if x > 0 { y = 1 } else { y = 0 }");
    }

    #[test]
    fn test_round_trip_while() {
        round_trip("while x < 10 { x = x + 1 }");
    }

    #[test]
    fn test_round_trip_for() {
        round_trip("for i in 0..10 { x = x + i }");
    }

    #[test]
    fn test_round_trip_print() {
        round_trip("print(1, 2, 3)");
    }

    #[test]
    fn test_round_trip_reverse_block() {
        round_trip("reverse { x += 10 y += 5 }");
    }

    #[test]
    fn test_round_trip_list() {
        round_trip("nums = [1, 2, 3]");
    }

    #[test]
    fn test_round_trip_tuple() {
        round_trip("point = (10, 20)");
    }

    #[test]
    fn test_round_trip_module() {
        round_trip("module M { fn f(x: Int): Int { return x } }");
    }

    #[test]
    fn test_round_trip_import() {
        round_trip("import Math");
    }

    #[test]
    fn test_round_trip_import_alias() {
        round_trip("import Math as M");
    }

    #[test]
    fn test_display_number_int() {
        assert_eq!(Number::Int(42).to_string(), "42");
    }

    #[test]
    // 3.14 here is the literal display string under test, not an approximation
    // of PI — the assertion checks the rendered text is exactly "3.14".
    #[allow(clippy::approx_constant)]
    fn test_display_number_float() {
        assert_eq!(Number::Float(3.14).to_string(), "3.14");
    }

    #[test]
    fn test_display_number_rational() {
        assert_eq!(Number::Rational(1, 3).to_string(), "1/3");
    }

    #[test]
    fn test_display_number_complex() {
        assert_eq!(Number::Complex(3.0, 4.0).to_string(), "3+4i");
    }

    #[test]
    fn test_display_number_hex() {
        assert_eq!(Number::Hex("0xFF".to_string()).to_string(), "0xFF");
    }

    #[test]
    fn test_display_number_binary() {
        assert_eq!(Number::Binary("0b1010".to_string()).to_string(), "0b1010");
    }

    #[test]
    fn test_display_purity() {
        assert_eq!(Purity::Pure.to_string(), "@pure");
        assert_eq!(Purity::Total.to_string(), "@total");
        assert_eq!(Purity::Impure.to_string(), "");
    }

    #[test]
    fn test_display_comparator() {
        assert_eq!(Comparator::Eq.to_string(), "==");
        assert_eq!(Comparator::Ne.to_string(), "!=");
        assert_eq!(Comparator::Lt.to_string(), "<");
        assert_eq!(Comparator::Le.to_string(), "<=");
        assert_eq!(Comparator::Gt.to_string(), ">");
        assert_eq!(Comparator::Ge.to_string(), ">=");
    }

    #[test]
    fn test_display_logical_op() {
        assert_eq!(LogicalOp::And.to_string(), "&&");
        assert_eq!(LogicalOp::Or.to_string(), "||");
    }

    #[test]
    fn test_custom_indent() {
        let printer = PrettyPrinter::with_indent(2);
        let func = FunctionDecl {
            name: "f".to_string(),
            params: vec![],
            return_type: Some(TypeAnnotation::Basic(BasicType::Int)),
            purity: Purity::Impure,
            echo_annotation: None,
            body: vec![ControlStmt::Return(Some(DataExpr::Number(Number::Int(0))))],
        };
        let rendered = printer.print_function(&func, 0);
        assert!(rendered.contains("  return 0"), "Should use 2-space indent");
    }
}
