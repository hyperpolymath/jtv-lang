<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# JtV Command Line Interface

The `jtv` CLI is the primary tool for working with Julia the Viper programs.

## Installation

```bash
cargo install jtzig
```

## Commands

### run

Execute a JtV program.

```bash
jtv run <file.jtv> [options]
```

**Options:**
- `--trace` - Show execution trace
- `--time` - Show execution time
- `--numbers <format>` - Number display format (decimal, hex, binary)
- `--stdin <file>` - Read input from file
- `--args <args...>` - Pass arguments to program

**Examples:**
```bash
jtv run main.jtv
jtv run main.jtv --trace
jtv run script.jtv --args 42 "hello"
```

### repl

Start the interactive Read-Eval-Print Loop.

```bash
jtv repl [options]
```

**Options:**
- `--load <file>` - Pre-load a JtV file
- `--no-history` - Disable command history
- `--numbers <format>` - Default number format

**REPL Commands:**
```
:help           Show help
:quit / :q      Exit REPL
:clear          Clear screen
:load <file>    Load a file
:reset          Reset state
:type <expr>    Show expression type
:trace <expr>   Trace expression evaluation
:history        Show command history
```

### check

Validate a JtV program without executing.

```bash
jtv check <file.jtv> [options]
```

**Options:**
- `--strict` - Enable all warnings as errors
- `--purity` - Check purity annotations
- `--types` - Show inferred types

**Examples:**
```bash
jtv check main.jtv
jtv check src/*.jtv --strict
```

### fmt

Format JtV source code.

```bash
jtv fmt <files...> [options]
```

**Options:**
- `--check` - Check formatting without modifying
- `--diff` - Show diff of changes
- `--write` - Write changes in-place (default)
- `--config <file>` - Use custom config

**Examples:**
```bash
jtv fmt src/
jtv fmt main.jtv --check
jtv fmt **/*.jtv --diff
```

### lint

Run the linter on JtV code.

```bash
jtv lint <files...> [options]
```

**Options:**
- `--fix` - Auto-fix issues where possible
- `--config <file>` - Custom lint config
- `--format <format>` - Output format (text, json, sarif)

**Examples:**
```bash
jtv lint src/
jtv lint main.jtv --fix
```

### test

Run JtV tests.

```bash
jtv test [options]
```

**Options:**
- `--filter <pattern>` - Filter tests by name
- `--verbose` - Verbose output
- `--coverage` - Generate coverage report
- `--parallel` - Run tests in parallel
- `--property` - Run property-based tests

**Examples:**
```bash
jtv test
jtv test --filter "math*"
jtv test --coverage --verbose
```

### build

Compile JtV to various targets.

```bash
jtv build <file.jtv> [options]
```

**Options:**
- `--target <target>` - Output target (wasm, native, bytecode)
- `--output <file>` - Output file path
- `--optimize` - Enable optimizations
- `--debug` - Include debug info

**Examples:**
```bash
jtv build main.jtv --target wasm
jtv build main.jtv --target native --optimize
```

### doc

Generate documentation.

```bash
jtv doc <files...> [options]
```

**Options:**
- `--output <dir>` - Output directory
- `--format <format>` - Output format (html, markdown)
- `--open` - Open in browser after generation

**Examples:**
```bash
jtv doc src/ --output docs/
jtv doc src/ --format html --open
```

### lsp

Start the Language Server Protocol server.

```bash
jtv lsp [options]
```

**Options:**
- `--stdio` - Use stdio transport (default)
- `--tcp <port>` - Use TCP transport
- `--log <file>` - Log to file

### new

Create a new JtV project.

```bash
jtv new <name> [options]
```

**Options:**
- `--lib` - Create a library project
- `--bin` - Create a binary project (default)
- `--template <name>` - Use a template

**Examples:**
```bash
jtv new my-project
jtv new my-lib --lib
```

## Configuration

### jtv.toml

Project configuration file:

```toml
[package]
name = "my-project"
version = "0.1.0"
authors = ["Your Name"]

[build]
target = "native"
optimize = true

[lint]
strict = true
deny = ["unused-variables", "shadowing"]

[fmt]
indent = 4
max-line-length = 100

[test]
parallel = true
coverage = true
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `JTV_HOME` | JtV installation directory | `~/.jtv` |
| `JTV_PATH` | Module search path | `.` |
| `JTV_LOG` | Log level | `warn` |
| `NO_COLOR` | Disable colored output | unset |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Runtime error |
| 2 | Parse error |
| 3 | Type error |
| 4 | Purity error |
| 5 | I/O error |
| 101 | Internal error |

## Shell Completion

### Bash
```bash
jtv completions bash > /etc/bash_completion.d/jtv
```

### Zsh
```bash
jtv completions zsh > ~/.zsh/completions/_jtv
```

### Fish
```bash
jtv completions fish > ~/.config/fish/completions/jtv.fish
```

## See Also

- [REPL](./REPL.md)
- [LSP](./LSP.md)
- [Formatter](./Formatter.md)
- [Linter](./Linter.md)
