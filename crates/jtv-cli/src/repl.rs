// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Julia the Viper Contributors
//
// Julia the Viper - Interactive REPL

use colored::*;
use jtv_core::{parse_program, Interpreter};
use rustyline::error::ReadlineError;
use rustyline::{Config, DefaultEditor, EditMode};

const HISTORY_FILE: &str = ".jtv_history";

pub struct Repl {
    interpreter: Interpreter,
    trace_enabled: bool,
    multiline_buffer: String,
    in_multiline: bool,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            trace_enabled: false,
            multiline_buffer: String::new(),
            in_multiline: false,
        }
    }

    pub fn run(&mut self) -> rustyline::Result<()> {
        self.print_banner();

        let config = Config::builder()
            .history_ignore_space(true)
            .edit_mode(EditMode::Emacs)
            .build();

        let mut rl = DefaultEditor::with_config(config)?;

        // Load history
        let history_path = self.get_history_path();
        let _ = rl.load_history(&history_path);

        loop {
            let prompt = if self.in_multiline {
                "... ".cyan().to_string()
            } else {
                "jtv> ".green().bold().to_string()
            };

            match rl.readline(&prompt) {
                Ok(line) => {
                    let trimmed = line.trim();

                    // Handle empty lines
                    if trimmed.is_empty() {
                        if self.in_multiline {
                            // Empty line in multiline mode: execute buffer
                            self.execute_multiline(&mut rl);
                        }
                        continue;
                    }

                    // Handle special commands
                    if !self.in_multiline && trimmed.starts_with(':') {
                        if self.handle_command(trimmed) {
                            continue;
                        } else {
                            // :quit was called
                            break;
                        }
                    }

                    // Check for multiline start (block or function)
                    if self.should_start_multiline(trimmed) {
                        self.in_multiline = true;
                        self.multiline_buffer = line.clone();
                        self.multiline_buffer.push('\n');
                        continue;
                    }

                    // In multiline mode, accumulate
                    if self.in_multiline {
                        self.multiline_buffer.push_str(&line);
                        self.multiline_buffer.push('\n');

                        // Check if block is complete
                        if self.is_block_complete() {
                            self.execute_multiline(&mut rl);
                        }
                        continue;
                    }

                    // Single line execution
                    let _ = rl.add_history_entry(&line);
                    self.execute(&line);
                }
                Err(ReadlineError::Interrupted) => {
                    if self.in_multiline {
                        println!("{}", "Cancelled multiline input".yellow());
                        self.in_multiline = false;
                        self.multiline_buffer.clear();
                    } else {
                        println!("Use :quit or Ctrl-D to exit");
                    }
                }
                Err(ReadlineError::Eof) => {
                    println!("\n{}", "Goodbye!".cyan());
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        // Save history
        let _ = rl.save_history(&history_path);

        Ok(())
    }

    fn print_banner(&self) {
        println!(
            "{}",
            "╔═══════════════════════════════════════════════════════════╗".cyan()
        );
        println!(
            "{}",
            "║         Julia the Viper - Interactive REPL                ║".cyan()
        );
        println!(
            "{}",
            "║    Harvard Architecture: Security through Grammar         ║".cyan()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════════╝".cyan()
        );
        println!();
        println!(
            "Type {} for commands, {} to exit",
            ":help".green(),
            ":quit".green()
        );
        println!();
    }

    fn handle_command(&mut self, cmd: &str) -> bool {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let arg = parts.get(1).map(|s| s.trim());

        match command {
            ":help" | ":h" => {
                self.print_help();
            }
            ":quit" | ":q" | ":exit" => {
                println!("{}", "Goodbye!".cyan());
                return false;
            }
            ":vars" | ":v" => {
                self.show_variables();
            }
            ":trace" | ":t" => {
                self.trace_enabled = !self.trace_enabled;
                if self.trace_enabled {
                    self.interpreter.enable_trace();
                    println!("{}", "Tracing enabled".green());
                } else {
                    self.interpreter.disable_trace();
                    println!("{}", "Tracing disabled".yellow());
                }
            }
            ":clear" | ":c" => {
                self.interpreter = Interpreter::new();
                if self.trace_enabled {
                    self.interpreter.enable_trace();
                }
                println!("{}", "State cleared".green());
            }
            ":load" | ":l" => {
                if let Some(path) = arg {
                    self.load_file(path);
                } else {
                    println!("{} Usage: :load <filename>", "Error:".red());
                }
            }
            ":ast" | ":a" => {
                if let Some(code) = arg {
                    self.show_ast(code);
                } else {
                    println!("{} Usage: :ast <expression>", "Error:".red());
                }
            }
            _ => {
                println!("{} Unknown command: {}", "Error:".red(), command);
                println!("Type :help for available commands");
            }
        }
        true
    }

    fn print_help(&self) {
        println!("{}", "=== REPL Commands ===".cyan().bold());
        println!();
        println!(
            "  {}  {:20} {}",
            ":help".green(),
            "(:h)",
            "Show this help message"
        );
        println!(
            "  {}  {:20} {}",
            ":quit".green(),
            "(:q, :exit)",
            "Exit the REPL"
        );
        println!(
            "  {}  {:20} {}",
            ":vars".green(),
            "(:v)",
            "Show all variables"
        );
        println!(
            "  {}  {:20} {}",
            ":trace".green(),
            "(:t)",
            "Toggle execution tracing"
        );
        println!(
            "  {}  {:20} {}",
            ":clear".green(),
            "(:c)",
            "Clear interpreter state"
        );
        println!(
            "  {} {:20} {}",
            ":load".green(),
            "<file>",
            "Load and execute a .jtv file"
        );
        println!(
            "  {}  {} {:20} {}",
            ":ast".green(),
            "<code>",
            "",
            "Show AST for code"
        );
        println!();
        println!("{}", "=== Language Examples ===".cyan().bold());
        println!();
        println!("  {}     {}", "x = 5".yellow(), "// Variable assignment");
        println!("  {}     {}", "y = x + 3".yellow(), "// Expression");
        println!("  {} {}", "if x > 0 { y = 1 }".yellow(), "// Conditional");
        println!("  {}   {}", "while x > 0 { x += -1 }".yellow(), "// Loop");
        println!(
            "  {}     {}",
            "reverse { x += 1 }".yellow(),
            "// Reversible block"
        );
        println!();
        println!("{}", "=== Data Types ===".cyan().bold());
        println!();
        println!("  Int, Float, Rational (3/4), Complex (1+2i)");
        println!("  Hex (0xFF), Binary (0b1010), Bool (true/false)");
        println!();
    }

    fn show_variables(&self) {
        println!("{}", "=== Variables ===".cyan().bold());
        let vars = self.interpreter.get_variables();
        if vars.is_empty() {
            println!("  (no variables defined)");
        } else {
            for (name, value) in vars {
                println!("  {} = {}", name.green(), value);
            }
        }
    }

    fn load_file(&mut self, path: &str) {
        match std::fs::read_to_string(path) {
            Ok(code) => {
                println!("{} {}", "Loading:".cyan(), path);
                self.execute(&code);
            }
            Err(e) => {
                println!("{} Failed to load '{}': {}", "Error:".red(), path, e);
            }
        }
    }

    fn show_ast(&self, code: &str) {
        match parse_program(code) {
            Ok(program) => {
                println!("{}", "=== AST ===".cyan().bold());
                println!("{:#?}", program);
            }
            Err(e) => {
                println!("{} {}", "Parse error:".red(), e);
            }
        }
    }

    fn execute(&mut self, code: &str) {
        match parse_program(code) {
            Ok(program) => {
                match self.interpreter.run(&program) {
                    Ok(_) => {
                        // Check if there's a result to display
                        if let Some(result) = self.interpreter.get_last_result() {
                            println!("{} {}", "=>".blue(), result);
                        }

                        // Show trace if enabled
                        if self.trace_enabled {
                            let trace = self.interpreter.get_trace();
                            if !trace.is_empty() {
                                println!("{}", "--- Trace ---".dimmed());
                                for entry in trace {
                                    println!("  {} {}", entry.stmt_type.yellow(), entry.line);
                                }
                            }
                        }
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

    fn should_start_multiline(&self, line: &str) -> bool {
        // Start multiline for blocks that aren't closed on the same line
        let opens = line.matches('{').count();
        let closes = line.matches('}').count();

        if opens > closes {
            return true;
        }

        // Also start multiline for function definitions
        if line.starts_with("fn ")
            || line.starts_with("@pure fn ")
            || line.starts_with("@total fn ")
        {
            return opens > closes;
        }

        false
    }

    fn is_block_complete(&self) -> bool {
        let opens = self.multiline_buffer.matches('{').count();
        let closes = self.multiline_buffer.matches('}').count();
        opens > 0 && opens == closes
    }

    fn execute_multiline(&mut self, rl: &mut DefaultEditor) {
        let code = std::mem::take(&mut self.multiline_buffer);
        self.in_multiline = false;

        let _ = rl.add_history_entry(code.trim());
        self.execute(&code);
    }

    fn get_history_path(&self) -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(HISTORY_FILE)
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}
