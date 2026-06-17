<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# Julia the Viper Wiki

Welcome to the Julia the Viper documentation wiki.

## What is JtV?

**Julia the Viper** is a Harvard Architecture programming language that makes code injection **grammatically impossible**. Named after mathematician Julia Robinson, it separates:

- **Data Language** (Total/Decidable): Addition-only expressions that always halt
- **Control Language** (Turing-complete): Imperative statements with loops and I/O

This architectural separation provides security guarantees at the grammar level, not runtime checks.

## Quick Navigation

### Getting Started
- [Installation](getting-started/Installation.md)
- [Hello World](getting-started/Hello-World.md)
- [IDE Setup](getting-started/IDE-Setup.md)

### Language Guide
- [Harvard Architecture](language/Harvard-Architecture.md)
- [Data Language](language/Data-Language.md)
- [Control Language](language/Control-Language.md)
- [Type System](language/Type-System.md)
- [Functions and Purity](language/Functions-Purity.md)
- [Modules](language/Modules.md)
- [Number Systems](language/Number-Systems.md)

### Tutorials
- [Basic Calculator](tutorials/Basic-Calculator.md)
- [Working with Lists](tutorials/Lists.md)
- [Smart Contracts](tutorials/Smart-Contracts.md)
- [Reversible Computing](tutorials/Reversible-Computing.md)

### Reference
- [Grammar (EBNF)](reference/Grammar-EBNF.md)
- [Standard Library](reference/Standard-Library.md)
- [CLI Commands](reference/CLI-Commands.md)
- [Error Messages](reference/Error-Messages.md)

### Tooling
- [CLI Tool](tooling/CLI.md)
- [REPL](tooling/REPL.md)
- [VS Code Extension](tooling/VS-Code.md)
- [Language Server](tooling/LSP.md)

### Internals
- [Compiler Architecture](internals/Compiler-Architecture.md)
- [AST Structure](internals/AST.md)
- [Type Checker](internals/Type-Checker.md)
- [Formal Proofs](internals/Formal-Proofs.md)

### Contributing
- [Development Setup](contributing/Development-Setup.md)
- [Code Style](contributing/Code-Style.md)
- [Testing](contributing/Testing.md)

## Why JtV?

### The Problem
Traditional languages allow code injection through:
- `eval()` in Python/JavaScript
- SQL string concatenation
- Shell command injection
- Template injection

### The Solution
JtV makes injection **grammatically impossible**:
- Data expressions can only contain addition
- No `eval()`, no string-to-code conversion
- Type system enforces separation
- Formally proven in Lean 4

### Key Features
- **7 Number Systems**: Int, Float, Rational, Complex, Hex, Binary, Symbolic
- **Purity Markers**: `@pure` (no I/O), `@total` (guaranteed termination)
- **Reversible Computing**: Forward/backward execution for quantum simulation
- **Formal Verification**: Lean 4 proofs of security properties

## Resources

- [GitHub Repository](https://github.com/hyperpolymath/julia-the-viper)
- [Technical Roadmap](../docs/TECHNICAL_ROADMAP.md)
- [CHANGELOG](../CHANGELOG.md)
- [License](../LICENSE.txt)
