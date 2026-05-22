// SPDX-License-Identifier: MPL-2.0
// Interactive debugger for Julia the Viper with reversibility inspection

#![forbid(unsafe_code)]
use colored::*;
use jtv_core::{parser::parse_program, Interpreter};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

struct Debugger {
    source_file: Option<PathBuf>,
    source_code: String,
    breakpoints: HashSet<usize>,
    current_line: usize,
    variables: HashMap<String, String>,
    call_stack: Vec<String>,
    paused: bool,
    interpreter: Interpreter,
}

impl Debugger {
    fn new() -> Self {
        let mut interpreter = Interpreter::new();
        interpreter.enable_output_capture();
        interpreter.enable_trace();

        Debugger {
            source_file: None,
            source_code: String::new(),
            breakpoints: HashSet::new(),
            current_line: 0,
            variables: HashMap::new(),
            call_stack: Vec::new(),
            paused: false,
            interpreter,
        }
    }

    fn load_file(&mut self, path: PathBuf) -> Result<()> {
        self.source_code = std::fs::read_to_string(&path)
            .map_err(|e| ReadlineError::Io(e))?;
        self.source_file = Some(path);
        println!("{}", "File loaded successfully".green());
        Ok(())
    }

    fn run_program(&mut self) {
        if self.source_code.is_empty() {
            println!("{}", "No source file loaded".red());
            return;
        }

        match parse_program(&self.source_code) {
            Ok(program) => {
                println!("{}", "Running program...".cyan());
                match self.interpreter.run(&program) {
                    Ok(_) => {
                        let output = self.interpreter.take_output();
                        if !output.is_empty() {
                            println!("\n{}", "Program output:".yellow());
                            for line in output {
                                println!("  {}", line);
                            }
                        }
                        println!("\n{}", "Program completed successfully".green());
                    }
                    Err(e) => {
                        println!("{} {}", "Runtime error:".red(), e);
                    }
                }
            }
            Err(e) => {
                println!("{} {}", "Parse error:".red(), e);
            }
        }
    }

    fn set_breakpoint(&mut self, line: usize) {
        self.breakpoints.insert(line);
        println!("{} {}", "Breakpoint set at line".green(), line);
    }

    fn delete_breakpoint(&mut self, line: usize) {
        if self.breakpoints.remove(&line) {
            println!("{} {}", "Breakpoint removed from line".green(), line);
        } else {
            println!("{}", "No breakpoint at that line".yellow());
        }
    }

    fn list_breakpoints(&self) {
        if self.breakpoints.is_empty() {
            println!("{}", "No breakpoints set".yellow());
        } else {
            println!("{}", "Breakpoints:".cyan());
            for line in &self.breakpoints {
                println!("  Line {}", line);
            }
        }
    }

    fn list_source(&self, start: usize, count: usize) {
        if self.source_code.is_empty() {
            println!("{}", "No source file loaded".red());
            return;
        }

        let lines: Vec<&str> = self.source_code.lines().collect();
        let end = (start + count).min(lines.len());

        for (i, line) in lines.iter().enumerate().skip(start).take(end - start) {
            let line_num = i + 1;
            let bp_marker = if self.breakpoints.contains(&line_num) {
                "●".red()
            } else {
                " ".normal()
            };
            println!("{} {:4} {}", bp_marker, line_num, line);
        }
    }

    fn print_variable(&self, name: &str) {
        match self.interpreter.get_variable(name) {
            Ok(value) => println!("{} = {}", name.cyan(), value),
            Err(_) => println!("{} {}", "Variable not found:".red(), name),
        }
    }

    fn list_variables(&self) {
        let vars = self.interpreter.get_variables();
        if vars.is_empty() {
            println!("{}", "No variables defined".yellow());
        } else {
            println!("{}", "Variables:".cyan());
            for (name, value) in vars {
                println!("  {} = {}", name.cyan(), value);
            }
        }
    }

    fn show_trace(&self) {
        let trace = self.interpreter.get_trace();
        if trace.is_empty() {
            println!("{}", "No trace available".yellow());
        } else {
            println!("{}", "Execution trace:".cyan());
            for entry in trace {
                println!("  {:?}", entry);
            }
        }
    }

    fn show_help(&self) {
        println!("\n{}", "Julia the Viper Debugger Commands:".bold().cyan());
        println!("  {}              - Run the loaded program", "run".green());
        println!("  {}        - Set breakpoint at line N", "break N".green());
        println!("  {}       - Delete breakpoint at line N", "delete N".green());
        println!("  {}             - List all breakpoints", "breakpoints".green());
        println!("  {}      - List source code (from line N, M lines)", "list [N] [M]".green());
        println!("  {}         - Print variable value", "print VAR".green());
        println!("  {}          - List all variables", "locals".green());
        println!("  {}            - Show execution trace", "trace".green());
        println!("  {}      - Load source file", "load FILE".green());
        println!("  {}            - Reset interpreter state", "reset".green());
        println!("  {}             - Show this help", "help".green());
        println!("  {}             - Exit debugger", "quit".green());
        println!();
    }
}

fn main() -> Result<()> {
    println!("{}", "Julia the Viper Interactive Debugger".bold().cyan());
    println!("{}", "Type 'help' for commands\n".yellow());

    let mut debugger = Debugger::new();
    let mut editor = DefaultEditor::new()?;

    // Check for source file argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        if let Err(e) = debugger.load_file(path) {
            println!("{} {}", "Failed to load file:".red(), e);
        }
    }

    loop {
        let prompt = "jtv-debug> ";
        match editor.readline(prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                editor.add_history_entry(line)?;

                let parts: Vec<&str> = line.split_whitespace().collect();
                match parts.get(0).map(|s| *s) {
                    Some("run") => debugger.run_program(),
                    Some("break") | Some("b") => {
                        if let Some(line_str) = parts.get(1) {
                            if let Ok(line_num) = line_str.parse::<usize>() {
                                debugger.set_breakpoint(line_num);
                            } else {
                                println!("{}", "Invalid line number".red());
                            }
                        } else {
                            println!("{}", "Usage: break N".yellow());
                        }
                    }
                    Some("delete") | Some("d") => {
                        if let Some(line_str) = parts.get(1) {
                            if let Ok(line_num) = line_str.parse::<usize>() {
                                debugger.delete_breakpoint(line_num);
                            } else {
                                println!("{}", "Invalid line number".red());
                            }
                        } else {
                            println!("{}", "Usage: delete N".yellow());
                        }
                    }
                    Some("breakpoints") | Some("bp") => debugger.list_breakpoints(),
                    Some("list") | Some("l") => {
                        let start = parts.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
                        let count = parts.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(20);
                        debugger.list_source(start, count);
                    }
                    Some("print") | Some("p") => {
                        if let Some(var) = parts.get(1) {
                            debugger.print_variable(var);
                        } else {
                            println!("{}", "Usage: print VAR".yellow());
                        }
                    }
                    Some("locals") => debugger.list_variables(),
                    Some("trace") | Some("t") => debugger.show_trace(),
                    Some("load") => {
                        if let Some(file) = parts.get(1) {
                            let path = PathBuf::from(file);
                            if let Err(e) = debugger.load_file(path) {
                                println!("{} {}", "Failed to load file:".red(), e);
                            }
                        } else {
                            println!("{}", "Usage: load FILE".yellow());
                        }
                    }
                    Some("reset") => {
                        debugger.interpreter.reset();
                        println!("{}", "Interpreter state reset".green());
                    }
                    Some("help") | Some("h") | Some("?") => debugger.show_help(),
                    Some("quit") | Some("q") | Some("exit") => {
                        println!("{}", "Exiting debugger".cyan());
                        break;
                    }
                    _ => println!("{} {}", "Unknown command:".red(), line),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C".yellow());
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Exiting debugger".cyan());
                break;
            }
            Err(err) => {
                println!("{} {:?}", "Error:".red(), err);
                break;
            }
        }
    }

    Ok(())
}
