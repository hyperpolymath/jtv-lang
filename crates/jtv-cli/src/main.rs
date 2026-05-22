// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Julia the Viper - Command Line Interface

// Allow some clippy lints for cleaner output
#![allow(clippy::wildcard_in_or_patterns)]
#![allow(clippy::print_literal)]
#![allow(clippy::collapsible_if)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::{Parser, Subcommand};
use colored::*;
use jtv_core::coproc::{CoprocEnv, resolve_coproc_blocks};
use jtv_core::coproc_lower::{lower_namespace, write_lowered};
use jtv_core::{format_code, parse_program, Interpreter, PurityChecker, TypeChecker};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

mod repl;
mod rsr_check;
use repl::Repl;
use rsr_check::RsrChecker;

#[derive(Parser)]
#[command(name = "jtv")]
#[command(about = "Julia the Viper - Harvard Architecture Language", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a JtV program
    Run {
        /// Path to the .jtv file (use '-' for stdin)
        file: String,

        /// Enable execution tracing
        #[arg(short, long)]
        trace: bool,

        /// Print variable values after execution
        #[arg(short, long)]
        vars: bool,
    },

    /// Parse a JtV file and display the AST
    Parse {
        /// Path to the .jtv file
        file: String,

        /// Output format (json or pretty)
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },

    /// Check a JtV file for errors without executing
    Check {
        /// Path to the .jtv file
        file: String,
    },

    /// Analyze legacy code for JtV extraction opportunities
    Analyze {
        /// Path to the legacy code file
        file: String,

        /// Language (python, javascript, ruby)
        #[arg(short, long, default_value = "javascript")]
        lang: String,
    },

    /// Display version and build information
    Version,

    /// Check RSR (Rhodium Standard Repository) compliance
    RsrCheck,

    /// Start the interactive REPL
    Repl,

    /// Format a JtV source file (prints to stdout; use shell redirection to overwrite)
    Fmt {
        /// Path to the .jtv file (use '-' for stdin)
        file: String,

        /// Write result back to the file instead of stdout
        #[arg(short, long)]
        write: bool,

        /// Exit with non-zero status if the file would change (check mode)
        #[arg(long)]
        check: bool,
    },

    /// Lower live extern coproc decls to Zig FFI + Idris2 ABI + C headers
    Lower {
        /// Path to the .jtv source file
        file: String,

        /// Path to the .pata gate-decision file
        #[arg(long)]
        pata: String,

        /// Target triple (e.g. riscv64gc-unknown-none-elf)
        #[arg(long)]
        target: String,

        /// Comma-separated ISA feature flags (e.g. v,zbb)
        #[arg(long, default_value = "")]
        features: String,

        /// Root directory for generated artefacts (default: current dir)
        #[arg(long, default_value = ".")]
        output_dir: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, trace, vars } => {
            if let Err(e) = run_file(&file, trace, vars) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Parse { file, format } => {
            if let Err(e) = parse_file(&file, &format) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Check { file } => {
            if let Err(e) = check_file(&file) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            } else {
                println!("{} No errors found", "✓".green().bold());
            }
        }
        Commands::Analyze { file, lang } => {
            eprintln!(
                "{} Analyzer not yet fully implemented",
                "Warning:".yellow().bold()
            );
            eprintln!(
                "Please use: deno run --allow-read packages/jtv-analyzer/src/main.ts {} {}",
                file, lang
            );
        }
        Commands::Version => {
            print_version();
        }
        Commands::RsrCheck => {
            let mut checker = RsrChecker::new();
            checker.check_all();
        }
        Commands::Repl => {
            let mut repl = Repl::new();
            if let Err(e) = repl.run() {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Fmt { file, write, check } => {
            if let Err(e) = fmt_file(&file, write, check) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Lower { file, pata, target, features, output_dir } => {
            let feats: Vec<&str> = if features.is_empty() {
                vec![]
            } else {
                features.split(',').map(|s| s.trim()).collect()
            };
            if let Err(e) = lower_files(&file, &pata, &target, &feats, &output_dir) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
    }
}

fn run_file(file_path: &str, trace: bool, show_vars: bool) -> Result<(), String> {
    let code = read_file(file_path)?;

    let program = parse_program(&code).map_err(|e| format!("Parse error: {}", e))?;

    // SECURITY: Run type checking before execution
    let mut type_checker = TypeChecker::new();
    type_checker
        .check_program(&program)
        .map_err(|e| format!("Type error: {}", e))?;

    // SECURITY: Run purity checking before execution
    let mut purity_checker = PurityChecker::new();
    purity_checker
        .check_program(&program)
        .map_err(|e| format!("Purity error: {}", e))?;

    let mut interpreter = Interpreter::new();

    if trace {
        interpreter.enable_trace();
    }

    interpreter
        .run(&program)
        .map_err(|e| format!("Runtime error: {}", e))?;

    if trace {
        println!("\n{}", "=== Execution Trace ===".cyan().bold());
        for entry in interpreter.get_trace() {
            println!("{}: {}", entry.stmt_type.yellow(), entry.line);
        }
    }

    if show_vars {
        println!("\n{}", "=== Variables ===".cyan().bold());
        for (name, value) in interpreter.get_variables() {
            println!("  {} = {}", name.green(), value);
        }
    }

    Ok(())
}

fn parse_file(file_path: &str, format: &str) -> Result<(), String> {
    let code = read_file(file_path)?;

    let program = parse_program(&code).map_err(|e| format!("Parse error: {}", e))?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&program).map_err(|e| e.to_string())?;
            println!("{}", json);
        }
        "sexpr" | "sexp" => {
            println!("{}", ast_to_sexpr(&program));
        }
        "pretty" | _ => {
            println!("{}", "=== Abstract Syntax Tree ===".cyan().bold());
            println!("{:#?}", program);
        }
    }

    Ok(())
}

fn format_file(file_path: &str, write_back: bool) -> Result<(), String> {
    let code = read_file(file_path)?;

    let formatted = format_code(&code)?;

    if write_back && file_path != "-" {
        fs::write(file_path, &formatted)
            .map_err(|e| format!("Failed to write file '{}': {}", file_path, e))?;
        println!("{} {}", "Formatted:".green().bold(), file_path);
    } else {
        print!("{}", formatted);
    }

    Ok(())
}

fn check_file(file_path: &str) -> Result<(), String> {
    let code = read_file(file_path)?;

    let program = parse_program(&code).map_err(|e| format!("Parse error: {}", e))?;

    // Type checking
    let mut type_checker = TypeChecker::new();
    type_checker
        .check_program(&program)
        .map_err(|e| format!("Type error: {}", e))?;

    // Purity checking
    let mut purity_checker = PurityChecker::new();
    purity_checker
        .check_program(&program)
        .map_err(|e| format!("Purity error: {}", e))?;

    Ok(())
}

fn fmt_file(file_path: &str, write: bool, check: bool) -> Result<(), String> {
    let original = read_file(file_path)?;
    let formatted = format_code(&original).map_err(|e| format!("Format error: {}", e))?;

    if check {
        if formatted == original {
            println!("{} {}", "✓".green().bold(), file_path);
        } else {
            eprintln!("{} {} would be reformatted", "✗".red().bold(), file_path);
            std::process::exit(1);
        }
        return Ok(());
    }

    if write {
        if file_path == "-" {
            return Err("--write cannot be used with stdin input".into());
        }
        fs::write(file_path, &formatted).map_err(|e| format!("Write error: {}", e))?;
        println!("{} {}", "✓".green().bold(), file_path);
    } else {
        print!("{}", formatted);
    }

    Ok(())
}

fn read_file(file_path: &str) -> Result<String, String> {
    if file_path == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| format!("Failed to read stdin: {}", e))?;
        Ok(buffer)
    } else {
        fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))
    }
}

/// Convert a JtV AST to S-expression representation for visualization
fn ast_to_sexpr(program: &jtv_core::Program) -> String {
    let mut out = String::new();
    out.push_str("(program");
    for stmt in &program.statements {
        out.push_str("\n  ");
        top_level_to_sexpr(stmt, &mut out, 2);
    }
    out.push(')');
    out
}

fn top_level_to_sexpr(tl: &jtv_core::TopLevel, out: &mut String, indent: usize) {
    match tl {
        jtv_core::TopLevel::Module(m) => {
            out.push_str(&format!("(module \"{}\"", m.name));
            for item in &m.body {
                out.push('\n');
                out.push_str(&" ".repeat(indent + 2));
                top_level_to_sexpr(item, out, indent + 2);
            }
            out.push(')');
        }
        jtv_core::TopLevel::Import(i) => {
            out.push_str(&format!("(import \"{}\"", i.path.join(".")));
            if let Some(alias) = &i.alias {
                out.push_str(&format!(" :as \"{}\"", alias));
            }
            out.push(')');
        }
        jtv_core::TopLevel::Function(f) => {
            out.push_str(&format!("(fn \"{}\" {:?}", f.name, f.purity));
            out.push_str("\n");
            out.push_str(&" ".repeat(indent + 2));
            out.push_str("(params");
            for p in &f.params {
                out.push_str(&format!(" \"{}\"", p.name));
            }
            out.push(')');
            for stmt in &f.body {
                out.push('\n');
                out.push_str(&" ".repeat(indent + 2));
                control_to_sexpr(stmt, out, indent + 2);
            }
            out.push(')');
        }
        jtv_core::TopLevel::Control(s) => {
            control_to_sexpr(s, out, indent);
        }
        jtv_core::TopLevel::ExternCoproc(b) => {
            out.push_str(&format!("(extern-coproc \"{}\")", b.gate_name));
        }
    }
}

fn control_to_sexpr(stmt: &jtv_core::ControlStmt, out: &mut String, indent: usize) {
    match stmt {
        jtv_core::ControlStmt::Assignment(a) => {
            out.push_str(&format!("(assign \"{}\" ", a.target));
            expr_to_sexpr(&a.value, out);
            out.push(')');
        }
        jtv_core::ControlStmt::If(i) => {
            out.push_str("(if ");
            cexpr_to_sexpr(&i.condition, out);
            out.push_str("\n");
            out.push_str(&" ".repeat(indent + 2));
            out.push_str("(then");
            for s in &i.then_branch {
                out.push(' ');
                control_to_sexpr(s, out, indent + 4);
            }
            out.push(')');
            if let Some(els) = &i.else_branch {
                out.push_str("\n");
                out.push_str(&" ".repeat(indent + 2));
                out.push_str("(else");
                for s in els {
                    out.push(' ');
                    control_to_sexpr(s, out, indent + 4);
                }
                out.push(')');
            }
            out.push(')');
        }
        jtv_core::ControlStmt::While(w) => {
            out.push_str("(while ");
            cexpr_to_sexpr(&w.condition, out);
            for s in &w.body {
                out.push('\n');
                out.push_str(&" ".repeat(indent + 2));
                control_to_sexpr(s, out, indent + 2);
            }
            out.push(')');
        }
        jtv_core::ControlStmt::For(f) => {
            out.push_str(&format!("(for \"{}\" ", f.variable));
            data_to_sexpr(&f.range.start, out);
            out.push_str(" .. ");
            data_to_sexpr(&f.range.end, out);
            for s in &f.body {
                out.push('\n');
                out.push_str(&" ".repeat(indent + 2));
                control_to_sexpr(s, out, indent + 2);
            }
            out.push(')');
        }
        jtv_core::ControlStmt::Return(val) => {
            out.push_str("(return");
            if let Some(v) = val {
                out.push(' ');
                data_to_sexpr(v, out);
            }
            out.push(')');
        }
        jtv_core::ControlStmt::Print(args) => {
            out.push_str("(print");
            for a in args {
                out.push(' ');
                data_to_sexpr(a, out);
            }
            out.push(')');
        }
        jtv_core::ControlStmt::ReverseBlock(r) => {
            out.push_str("(reverse");
            for s in &r.body {
                out.push('\n');
                out.push_str(&" ".repeat(indent + 2));
                match s {
                    jtv_core::ReversibleStmt::AddAssign(v, e) => {
                        out.push_str(&format!("(+= \"{}\" ", v));
                        data_to_sexpr(e, out);
                        out.push(')');
                    }
                    jtv_core::ReversibleStmt::SubAssign(v, e) => {
                        out.push_str(&format!("(-= \"{}\" ", v));
                        data_to_sexpr(e, out);
                        out.push(')');
                    }
                    jtv_core::ReversibleStmt::If(i) => {
                        control_to_sexpr(&jtv_core::ControlStmt::If(i.clone()), out, indent + 2);
                    }
                }
            }
            out.push(')');
        }
        jtv_core::ControlStmt::ReversibleBlock(rb) => {
            out.push_str("(reversible");
            for s in &rb.body {
                out.push('\n');
                out.push_str(&" ".repeat(indent + 2));
                match s {
                    jtv_core::ReversibleStmt::AddAssign(v, e) => {
                        out.push_str(&format!("(+= \"{}\" ", v));
                        data_to_sexpr(e, out);
                        out.push(')');
                    }
                    jtv_core::ReversibleStmt::SubAssign(v, e) => {
                        out.push_str(&format!("(-= \"{}\" ", v));
                        data_to_sexpr(e, out);
                        out.push(')');
                    }
                    jtv_core::ReversibleStmt::If(i) => {
                        control_to_sexpr(&jtv_core::ControlStmt::If(i.clone()), out, indent + 2);
                    }
                }
            }
            if let Some(tok) = &rb.token_binding {
                out.push_str(&format!(" -> \"{}\"", tok));
            }
            out.push(')');
        }
        jtv_core::ControlStmt::ReverseToken(tok) => {
            out.push_str(&format!("(reverse-token \"{}\")", tok));
        }
        jtv_core::ControlStmt::AbandonToken(tok) => {
            out.push_str(&format!("(abandon-token \"{}\")", tok));
        }
        jtv_core::ControlStmt::Block(stmts) => {
            out.push_str("(block");
            for s in stmts {
                out.push('\n');
                out.push_str(&" ".repeat(indent + 2));
                control_to_sexpr(s, out, indent + 2);
            }
            out.push(')');
        }
    }
}

fn expr_to_sexpr(expr: &jtv_core::Expr, out: &mut String) {
    match expr {
        jtv_core::Expr::Data(d) => data_to_sexpr(d, out),
        jtv_core::Expr::Control(c) => cexpr_to_sexpr(c, out),
    }
}

fn data_to_sexpr(expr: &jtv_core::DataExpr, out: &mut String) {
    match expr {
        jtv_core::DataExpr::Number(n) => {
            match n {
                jtv_core::Number::Int(i) => out.push_str(&format!("{}", i)),
                jtv_core::Number::Float(f) => out.push_str(&format!("{}", f)),
                jtv_core::Number::Rational(n, d) => out.push_str(&format!("(rational {} {})", n, d)),
                jtv_core::Number::Complex(r, i) => out.push_str(&format!("(complex {} {})", r, i)),
                jtv_core::Number::Hex(s) => out.push_str(&format!("(hex \"{}\")", s)),
                jtv_core::Number::Binary(s) => out.push_str(&format!("(binary \"{}\")", s)),
                jtv_core::Number::Symbolic(s) => out.push_str(&format!("(sym \"{}\")", s)),
            }
        }
        jtv_core::DataExpr::StringLit(s) => out.push_str(&format!("(str \"{}\")", s)),
        jtv_core::DataExpr::Identifier(name) => out.push_str(&format!("(id \"{}\")", name)),
        jtv_core::DataExpr::Add(l, r) => {
            out.push_str("(+ ");
            data_to_sexpr(l, out);
            out.push(' ');
            data_to_sexpr(r, out);
            out.push(')');
        }
        jtv_core::DataExpr::Negate(e) => {
            out.push_str("(- ");
            data_to_sexpr(e, out);
            out.push(')');
        }
        jtv_core::DataExpr::FunctionCall(fc) => {
            out.push_str(&format!("(call \"{}\"", fc.qualified_name()));
            for a in &fc.args {
                out.push(' ');
                data_to_sexpr(a, out);
            }
            out.push(')');
        }
        jtv_core::DataExpr::List(items) => {
            out.push_str("(list");
            for item in items {
                out.push(' ');
                data_to_sexpr(item, out);
            }
            out.push(')');
        }
        jtv_core::DataExpr::Tuple(items) => {
            out.push_str("(tuple");
            for item in items {
                out.push(' ');
                data_to_sexpr(item, out);
            }
            out.push(')');
        }
    }
}

fn cexpr_to_sexpr(expr: &jtv_core::ControlExpr, out: &mut String) {
    match expr {
        jtv_core::ControlExpr::Data(d) => data_to_sexpr(d, out),
        jtv_core::ControlExpr::Comparison(l, op, r) => {
            let op_str = match op {
                jtv_core::Comparator::Eq => "==",
                jtv_core::Comparator::Ne => "!=",
                jtv_core::Comparator::Lt => "<",
                jtv_core::Comparator::Le => "<=",
                jtv_core::Comparator::Gt => ">",
                jtv_core::Comparator::Ge => ">=",
            };
            out.push_str(&format!("({} ", op_str));
            data_to_sexpr(l, out);
            out.push(' ');
            data_to_sexpr(r, out);
            out.push(')');
        }
        jtv_core::ControlExpr::Logical(l, op, r) => {
            let op_str = match op {
                jtv_core::LogicalOp::And => "and",
                jtv_core::LogicalOp::Or => "or",
            };
            out.push_str(&format!("({} ", op_str));
            cexpr_to_sexpr(l, out);
            out.push(' ');
            cexpr_to_sexpr(r, out);
            out.push(')');
        }
        jtv_core::ControlExpr::Not(e) => {
            out.push_str("(not ");
            cexpr_to_sexpr(e, out);
            out.push(')');
        }
    }
}

fn print_version() {
    println!("{}", "Julia the Viper".cyan().bold());
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Build: {}", "Rust");
    println!();
    println!("{}", "Harvard Architecture Language".green());
    println!("Makes code injection grammatically impossible");
    println!();
    println!("Repository: https://github.com/Hyperpolymath/julia-the-viper");
    println!("Documentation: https://docs.julia-viper.dev");
}

/// Lower live extern coproc decls to Zig FFI + Idris2 ABI + C headers.
fn lower_files(
    jtv_path: &str,
    pata_path: &str,
    target: &str,
    features: &[&str],
    output_dir: &str,
) -> Result<(), String> {
    let src = fs::read_to_string(jtv_path)
        .map_err(|e| format!("cannot read {}: {}", jtv_path, e))?;
    let pata_src = fs::read_to_string(pata_path)
        .map_err(|e| format!("cannot read {}: {}", pata_path, e))?;

    let prog = parse_program(&src).map_err(|e| e.to_string())?;
    let env = CoprocEnv::from_triple(target, features);
    let (_, ns) = resolve_coproc_blocks(prog, &env, Some(&pata_src))
        .map_err(|e| e.to_string())?;

    let gates = lower_namespace(&ns);
    if gates.is_empty() {
        println!("{} no live extern coproc gates — nothing to lower", "note:".yellow());
        return Ok(());
    }

    let root = Path::new(output_dir);
    write_lowered(&gates, root).map_err(|e| format!("write error: {}", e))?;

    for gate in &gates {
        println!("{} {}", "lowered".green().bold(), gate.gate_name);
        println!("  zig   → {}/{}", output_dir, gate.zig_path());
        println!("  idris → {}/{}", output_dir, gate.idris2_path());
        println!("  C     → {}/{}", output_dir, gate.c_header_path());
    }
    Ok(())
}
