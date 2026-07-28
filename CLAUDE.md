# JtV

## Project Overview

**JtV** is a Harvard Architecture programming language designed as a "universal extender" to fix code injection vulnerabilities in legacy systems (Python/PHP/JS). The original name — *Julia the Viper* — honored mathematician **Julia Robinson** while punning on "adder" (snake + addition); it is now just **JtV** so the "Julia" is no longer mistaken for the Julia language (see *Etymology & Humor* below).

### The Core Insight

JtV grammatically separates **Control Language** (Turing-complete, imperative) from **Data Language** (Total/provably halting, addition-only). This makes code injection **grammatically impossible** - not a runtime check, but a fundamental architectural guarantee.

## Current Repository State

This repository currently contains the conceptual foundation:

```
jtv/
├── README.md           # Project tagline: "It's basically the same thing as an adder"
├── julia-viper         # Pseudocode showing the humorous starting point
├── LICENSE             # GPL-3.0
├── .gitignore          # Git ignore rules
└── CLAUDE.md           # This file - project vision and handover doc
```

**Note**: The full implementation (jtv-core Rust crate, examples, v2 specification) may exist in other branches or repositories. This document describes the complete vision.

## Architecture & Security Model

### Harvard Architecture Enforcement

- **Control Language**: Turing-complete, imperative, handles loops/conditionals/IO
- **Data Language**: Total (guaranteed halting), handles expressions and calculations
- **Critical Rule**: Data expressions CANNOT contain control flow
- **Result**: Code injection becomes grammatically impossible, not just "best practice"

### Why Addition-Only?

1. **Universal operation** across all number systems (int, float, rational, complex, hex, binary, symbolic)
2. **Enables v2 reversibility**: Addition inverts to subtraction in `reverse` blocks (quantum simulation)
3. **With Control loops**: Turing-complete (can implement any operation)
4. **Without Control loops**: Total (guaranteed to halt)

### Pure Function Rule (v2)

- Only **Pure Data Functions** (no loops/IO) are callable in Data context
- Compiler MUST enforce this - it's not a warning, it's a hard requirement
- Prevents side effects from breaking the Totality guarantee

## Version Evolution

### v1 (Alpha 1) - FOUNDATION

Full Rust implementation in `jtv-core/`:

- **Parser**: nom combinators for Control + Data languages
- **Interpreter**: 7 number systems (int, float, rational, complex, hex, binary, symbolic)
- **Instrumentation API**: Execution traces for visualization
- **Examples**: 5 programs in `examples/`
- **Status**: COMPLETED ✅

**Critical**: v1 MUST be mastered before approaching v2 (see `GRAMMAR_EVOLUTION.md`)

### v2 (Beta 1) - QUANTUM LEAP

Specification in `docs/`:

- **EBNF grammar**: Reversibility, functions, modules
- **Reversible computing**: `reverse` blocks invert operations (+ becomes -)
- **Quantum vision**: Simulate unitary transformations, Bennett's trick, Grover's/Shor's algorithms
- **Landauer's principle**: Reversibility = thermodynamically efficient computation
- **Status**: Specification only, implementation pending

**Critical**: v2 preserves v1's Totality guarantee via Pure Function enforcement

## Key Files & Documentation

Must-read files for context:

- `README.adoc` - Main project README
- `STATUS.adoc` - Current implementation status (canonical)
- `docs/GRAMMAR_EVOLUTION.md` - v1 vs v2 separation rationale
- `docs/QUANTUM_VISION.md` - Quantum computing abstraction
- `Justfile` - Build system (NOT Makefile)
- `Cargo.lock` - Committed for reproducibility
- `target/` - In .gitignore (build artifacts)

## Development Roadmap

### Next Steps (Priority Order)

1. **WASM Compilation**: `just build-wasm` using wasm-pack
2. **ReScript PWA**: Scaffold with Vite
3. **Router Visualization** (THE KILLER DEMO):
   - Animate Control (blue) vs Data (red) channel separation
   - Show "bridge" when data results cross to control variables
   - This is the pedagogical key to understanding JtV
4. **Monaco Editor**: JtV syntax highlighting
5. **Number System Explorer**: Showcase rationals, complex, symbolic
6. **v2 Implementation**: Parser extension, purity checker, reverse semantics

### Critical Design Principles

1. **Playground must teach v1 first, v2 as quantum leap** - No conceptual conflation
2. **Show, don't tell** - Visual demos beat documentation
3. **Security is grammar, not runtime** - Emphasize the architectural guarantee
4. **Quantum connection is not hype** - Reversibility genuinely simulates quantum ops

## For AI Assistants Working on This Codebase

### What JtV IS

- A security-focused language with formal guarantees
- A pedagogical tool for teaching Harvard Architecture
- A quantum computing abstraction layer (v2)
- Named after Julia Robinson (mathematician, Hilbert's 10th problem)
- Playfully named "Viper" (snake) because it's addition-focused ("adder")

### What JtV IS NOT

- Just another calculator (the `julia-viper` pseudocode is the humble origin story)
- A joke project (the puns are real, but the security model is serious)
- Security by obscurity (it's security by grammatical impossibility)

### Common Gotchas

- **Don't conflate v1 and v2**: They are distinct evolutionary stages
- **Addition-only seems limiting**: It's universal with Control loops
- **"Why not just use Rust?"**: Because legacy systems need retrofitting
- **Reversibility seems esoteric**: It enables quantum algorithms AND thermodynamic efficiency

### When to Consult the Handover

If you see references to:
- Harvard Architecture separation
- Control vs Data languages
- Totality guarantees
- Reversible computing or quantum vision
- Pure Function enforcement
- The 7 number systems

...then this is the real JtV project, not just the simple pseudocode starter.

## Etymology & Humor

**JtV was formerly named *Julia the Viper*.** It now goes by **JtV** so the
"Julia" in the title is no longer mistaken — by people and bots alike — for the
Julia programming language. The wordplay behind the original name lives on:

- **Julia Robinson**: Mathematician who solved Hilbert's 10th problem (the "Julia")
- **Viper**: A snake (like "adder", which is both a snake and a calculator)
- **"It's basically the same thing as an adder"**: Humble origin story
- **Addition-only**: Seems limited, actually universal

Maintain this lighthearted spirit while ensuring rigorous implementation.

## Build System Notes

- Uses **Justfile** (not Makefile) - run `just --list` for commands
- `Cargo.lock` is committed for reproducibility
- `target/` directory in .gitignore
- WASM build: `just build-wasm` (uses wasm-pack)

## Testing Philosophy

- **v1 Tests**: Verify Totality (Data Language must halt), number system correctness
- **v2 Tests**: Verify reversibility (forward then reverse = identity), purity enforcement
- **Integration Tests**: Router Visualization demos must work in PWA
- **Security Tests**: Attempt code injection, verify grammatical rejection

## Contact & Contribution

When extending JtV:

1. Master v1 before touching v2
2. Preserve the Totality guarantee
3. Keep the Harvard Architecture strict
4. Document with humor, implement with rigor
5. Visualize concepts (Router Visualization is the killer demo)

Remember: Code injection isn't prevented by careful programming - it's prevented by making it grammatically impossible to express.
