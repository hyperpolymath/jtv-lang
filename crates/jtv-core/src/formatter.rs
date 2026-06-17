// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Julia the Viper Contributors
//
// Julia the Viper - Code Formatter

use crate::ast::*;

/// Configuration options for the formatter
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Number of spaces per indentation level
    pub indent_size: usize,
    /// Maximum line length before wrapping
    pub max_line_length: usize,
    /// Add blank line between top-level items
    pub blank_lines_between_items: bool,
    /// Use spaces around operators
    pub spaces_around_operators: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            max_line_length: 100,
            blank_lines_between_items: true,
            spaces_around_operators: true,
        }
    }
}

/// Code formatter for JtV programs
pub struct Formatter {
    config: FormatConfig,
    output: String,
    indent_level: usize,
}

impl Formatter {
    pub fn new() -> Self {
        Self::with_config(FormatConfig::default())
    }

    pub fn with_config(config: FormatConfig) -> Self {
        Formatter {
            config,
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Format a complete program
    pub fn format_program(&mut self, program: &Program) -> String {
        self.output.clear();
        self.indent_level = 0;

        for (i, stmt) in program.statements.iter().enumerate() {
            if i > 0 && self.config.blank_lines_between_items {
                self.output.push('\n');
            }
            self.format_top_level(stmt);
        }

        self.output.trim_end().to_string() + "\n"
    }

    fn format_top_level(&mut self, top_level: &TopLevel) {
        match top_level {
            TopLevel::Module(module) => self.format_module(module),
            TopLevel::Import(import) => self.format_import(import),
            TopLevel::Function(func) => self.format_function(func),
            TopLevel::Control(stmt) => {
                self.format_control_stmt(stmt);
                self.output.push('\n');
            }
            TopLevel::ExternCoproc(block) => {
                self.write_indent();
                self.output.push_str(&format!(
                    "extern coproc {} {{ /* {} item(s) */ }}\n",
                    block.gate_name,
                    block.items.len()
                ));
            }
        }
    }

    fn format_module(&mut self, module: &ModuleDecl) {
        self.write_indent();
        self.output
            .push_str(&format!("module {} {{\n", module.name));
        self.indent_level += 1;

        for stmt in &module.body {
            self.format_top_level(stmt);
        }

        self.indent_level -= 1;
        self.write_indent();
        self.output.push_str("}\n");
    }

    fn format_import(&mut self, import: &ImportStmt) {
        self.write_indent();
        self.output.push_str("import ");
        self.output.push_str(&import.path.join("/"));
        if let Some(alias) = &import.alias {
            self.output.push_str(" as ");
            self.output.push_str(alias);
        }
        self.output.push('\n');
    }

    fn format_function(&mut self, func: &FunctionDecl) {
        self.write_indent();

        // Purity annotation
        match &func.purity {
            Purity::Total => self.output.push_str("@total "),
            Purity::Pure => self.output.push_str("@pure "),
            Purity::Impure => {}
        }

        // Function signature
        self.output.push_str("fn ");
        self.output.push_str(&func.name);
        self.output.push('(');

        // Parameters
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&param.name);
            if let Some(ty) = &param.type_annotation {
                self.output.push_str(": ");
                self.format_type_annotation(ty);
            }
        }

        self.output.push(')');

        // Return type
        if let Some(ret_type) = &func.return_type {
            self.output.push_str(": ");
            self.format_type_annotation(ret_type);
        }

        self.output.push_str(" {\n");
        self.indent_level += 1;

        // Function body
        for stmt in &func.body {
            self.format_control_stmt(stmt);
            self.output.push('\n');
        }

        self.indent_level -= 1;
        self.write_indent();
        self.output.push_str("}\n");
    }

    fn format_type_annotation(&mut self, ty: &TypeAnnotation) {
        match ty {
            TypeAnnotation::Basic(basic) => {
                self.output.push_str(match basic {
                    BasicType::Int => "Int",
                    BasicType::Float => "Float",
                    BasicType::Rational => "Rational",
                    BasicType::Complex => "Complex",
                    BasicType::Hex => "Hex",
                    BasicType::Binary => "Binary",
                    BasicType::Symbolic => "Symbolic",
                    BasicType::Bool => "Bool",
                    BasicType::String => "String",
                });
            }
            TypeAnnotation::List(inner) => {
                self.output.push_str("List<");
                self.format_type_annotation(inner);
                self.output.push('>');
            }
            TypeAnnotation::Tuple(types) => {
                self.output.push('(');
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_type_annotation(t);
                }
                self.output.push(')');
            }
            TypeAnnotation::Function(params, ret) => {
                self.output.push_str("Fn(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_type_annotation(p);
                }
                self.output.push_str(") -> ");
                self.format_type_annotation(ret);
            }
        }
    }

    fn format_control_stmt(&mut self, stmt: &ControlStmt) {
        self.write_indent();
        match stmt {
            ControlStmt::Assignment(assign) => {
                self.output.push_str(&assign.target);
                if self.config.spaces_around_operators {
                    self.output.push_str(" = ");
                } else {
                    self.output.push('=');
                }
                self.format_expr(&assign.value);
            }
            ControlStmt::If(if_stmt) => {
                self.output.push_str("if ");
                self.format_control_expr(&if_stmt.condition);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for s in &if_stmt.then_branch {
                    self.format_control_stmt(s);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.output.push_str(" else {\n");
                    self.indent_level += 1;
                    for s in else_branch {
                        self.format_control_stmt(s);
                        self.output.push('\n');
                    }
                    self.indent_level -= 1;
                    self.write_indent();
                    self.output.push('}');
                }
            }
            ControlStmt::While(while_stmt) => {
                self.output.push_str("while ");
                self.format_control_expr(&while_stmt.condition);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for s in &while_stmt.body {
                    self.format_control_stmt(s);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
            }
            ControlStmt::For(for_stmt) => {
                self.output.push_str("for ");
                self.output.push_str(&for_stmt.variable);
                self.output.push_str(" in ");
                self.format_data_expr(&for_stmt.range.start);
                self.output.push_str("..");
                self.format_data_expr(&for_stmt.range.end);
                if let Some(step) = &for_stmt.range.step {
                    self.output.push_str("..");
                    self.format_data_expr(step);
                }
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for s in &for_stmt.body {
                    self.format_control_stmt(s);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
            }
            ControlStmt::Return(expr) => {
                self.output.push_str("return");
                if let Some(e) = expr {
                    self.output.push(' ');
                    self.format_data_expr(e);
                }
            }
            ControlStmt::Print(exprs) => {
                self.output.push_str("print(");
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_data_expr(e);
                }
                self.output.push(')');
            }
            ControlStmt::ReverseBlock(block) => {
                self.output.push_str("reverse {\n");
                self.indent_level += 1;
                for s in &block.body {
                    self.format_reversible_stmt(s);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
            }
            ControlStmt::ReversibleBlock(rb) => {
                // `reversible { ... } -> tok` (the `-> tok` binding is optional).
                self.output.push_str("reversible {\n");
                self.indent_level += 1;
                for s in &rb.body {
                    self.format_reversible_stmt(s);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
                if let Some(tok) = &rb.token_binding {
                    self.output.push_str(" -> ");
                    self.output.push_str(tok);
                }
            }
            ControlStmt::ReverseToken(tok) => {
                self.output.push_str("reverse ");
                self.output.push_str(tok);
            }
            ControlStmt::AbandonToken(tok) => {
                self.output.push_str("abandon ");
                self.output.push_str(tok);
            }
            ControlStmt::Block(stmts) => {
                self.output.push_str("{\n");
                self.indent_level += 1;
                for s in stmts {
                    self.format_control_stmt(s);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');
            }
        }
    }

    fn format_reversible_stmt(&mut self, stmt: &ReversibleStmt) {
        self.write_indent();
        match stmt {
            ReversibleStmt::AddAssign(target, expr) => {
                self.output.push_str(target);
                if self.config.spaces_around_operators {
                    self.output.push_str(" += ");
                } else {
                    self.output.push_str("+=");
                }
                self.format_data_expr(expr);
            }
            ReversibleStmt::SubAssign(target, expr) => {
                self.output.push_str(target);
                if self.config.spaces_around_operators {
                    self.output.push_str(" -= ");
                } else {
                    self.output.push_str("-=");
                }
                self.format_data_expr(expr);
            }
            ReversibleStmt::If(if_stmt) => {
                self.output.push_str("if ");
                self.format_control_expr(&if_stmt.condition);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for s in &if_stmt.then_branch {
                    self.format_control_stmt(s);
                    self.output.push('\n');
                }
                self.indent_level -= 1;
                self.write_indent();
                self.output.push('}');

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.output.push_str(" else {\n");
                    self.indent_level += 1;
                    for s in else_branch {
                        self.format_control_stmt(s);
                        self.output.push('\n');
                    }
                    self.indent_level -= 1;
                    self.write_indent();
                    self.output.push('}');
                }
            }
        }
    }

    fn format_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Data(data) => self.format_data_expr(data),
            Expr::Control(ctrl) => self.format_control_expr(ctrl),
        }
    }

    fn format_control_expr(&mut self, expr: &ControlExpr) {
        match expr {
            ControlExpr::Data(data) => self.format_data_expr(data),
            ControlExpr::Comparison(left, op, right) => {
                self.format_data_expr(left);
                let op_str = match op {
                    Comparator::Eq => "==",
                    Comparator::Ne => "!=",
                    Comparator::Lt => "<",
                    Comparator::Le => "<=",
                    Comparator::Gt => ">",
                    Comparator::Ge => ">=",
                };
                if self.config.spaces_around_operators {
                    self.output.push_str(&format!(" {} ", op_str));
                } else {
                    self.output.push_str(op_str);
                }
                self.format_data_expr(right);
            }
            ControlExpr::Logical(left, op, right) => {
                self.format_control_expr(left);
                let op_str = match op {
                    LogicalOp::And => "&&",
                    LogicalOp::Or => "||",
                };
                if self.config.spaces_around_operators {
                    self.output.push_str(&format!(" {} ", op_str));
                } else {
                    self.output.push_str(op_str);
                }
                self.format_control_expr(right);
            }
            ControlExpr::Not(inner) => {
                self.output.push('!');
                self.format_control_expr(inner);
            }
        }
    }

    fn format_data_expr(&mut self, expr: &DataExpr) {
        match expr {
            DataExpr::Number(num) => self.format_number(num),
            DataExpr::StringLit(s) => {
                self.output.push('"');
                self.output.push_str(s);
                self.output.push('"');
            }
            DataExpr::Identifier(name) => self.output.push_str(name),
            DataExpr::Add(left, right) => {
                self.format_data_expr(left);
                if self.config.spaces_around_operators {
                    self.output.push_str(" + ");
                } else {
                    self.output.push('+');
                }
                self.format_data_expr(right);
            }
            DataExpr::Negate(inner) => {
                self.output.push('-');
                self.format_data_expr(inner);
            }
            DataExpr::FunctionCall(call) => {
                self.output.push_str(&call.name);
                self.output.push('(');
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_data_expr(arg);
                }
                self.output.push(')');
            }
            DataExpr::List(elements) => {
                self.output.push('[');
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_data_expr(e);
                }
                self.output.push(']');
            }
            DataExpr::Tuple(elements) => {
                self.output.push('(');
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.format_data_expr(e);
                }
                self.output.push(')');
            }
        }
    }

    fn format_number(&mut self, num: &Number) {
        match num {
            Number::Int(n) => self.output.push_str(&n.to_string()),
            Number::Float(f) => {
                let s = f.to_string();
                if s.contains('.') {
                    self.output.push_str(&s);
                } else {
                    self.output.push_str(&format!("{}.0", s));
                }
            }
            Number::Rational(num, denom) => {
                self.output.push_str(&format!("{}/{}", num, denom));
            }
            Number::Complex(re, im) => {
                if *re != 0.0 {
                    self.output.push_str(&format!("{}", re));
                    if *im >= 0.0 {
                        self.output.push('+');
                    }
                }
                self.output.push_str(&format!("{}i", im));
            }
            Number::Hex(s) => {
                // Hex values are stored as strings (e.g., "0xFF")
                self.output.push_str(s);
            }
            Number::Binary(s) => {
                // Binary values are stored as strings (e.g., "0b1010")
                self.output.push_str(s);
            }
            Number::Symbolic(s) => {
                // Symbolic literals use '#' prefix (e.g., #x, #pi)
                self.output.push_str(&format!("#{}", s));
            }
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            for _ in 0..self.config.indent_size {
                self.output.push(' ');
            }
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to format code
pub fn format_code(code: &str) -> Result<String, String> {
    use crate::parser::parse_program;
    let program = parse_program(code).map_err(|e| format!("Parse error: {}", e))?;
    let mut formatter = Formatter::new();
    Ok(formatter.format_program(&program))
}

/// Convenience function to format code with custom config
pub fn format_code_with_config(code: &str, config: FormatConfig) -> Result<String, String> {
    use crate::parser::parse_program;
    let program = parse_program(code).map_err(|e| format!("Parse error: {}", e))?;
    let mut formatter = Formatter::with_config(config);
    Ok(formatter.format_program(&program))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_assignment() {
        let code = "x=5+3";
        let formatted = format_code(code).unwrap();
        assert_eq!(formatted, "x = 5 + 3\n");
    }

    #[test]
    fn test_format_function() {
        let code = "fn add(a:Int,b:Int):Int{return a+b}";
        let formatted = format_code(code).unwrap();
        assert!(formatted.contains("fn add(a: Int, b: Int): Int {"));
        assert!(formatted.contains("return a + b"));
    }

    #[test]
    fn test_format_if_statement() {
        let code = "if x>0{y=1}";
        let formatted = format_code(code).unwrap();
        assert!(formatted.contains("if x > 0 {"));
        assert!(formatted.contains("y = 1"));
    }

    #[test]
    fn test_format_pure_function() {
        let code = "@pure fn double(x:Int):Int{return x+x}";
        let formatted = format_code(code).unwrap();
        assert!(formatted.contains("@pure fn double"));
    }

    #[test]
    fn test_format_reverse_block() {
        let code = "reverse{x+=5}";
        let formatted = format_code(code).unwrap();
        assert!(formatted.contains("reverse {"));
        assert!(formatted.contains("x += 5"));
    }

    #[test]
    fn test_format_list() {
        let code = "nums=[1,2,3,4,5]";
        let formatted = format_code(code).unwrap();
        assert_eq!(formatted, "nums = [1, 2, 3, 4, 5]\n");
    }

    #[test]
    fn test_format_for_loop() {
        let code = "for i in 0..10{x=x+i}";
        let formatted = format_code(code).unwrap();
        assert!(formatted.contains("for i in 0..10 {"));
    }

    #[test]
    fn test_config_no_spaces() {
        let code = "x = 5 + 3";
        let config = FormatConfig {
            spaces_around_operators: false,
            ..Default::default()
        };
        let formatted = format_code_with_config(code, config).unwrap();
        assert_eq!(formatted, "x=5+3\n");
    }

    #[test]
    fn test_format_while_loop() {
        let code = "while x>0{x=x+-1}";
        let formatted = format_code(code).unwrap();
        assert!(formatted.contains("while x > 0 {"));
        assert!(formatted.contains("x = x + -1"));
    }
}
