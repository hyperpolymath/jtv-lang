<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> -->

# TOPOLOGY.md — julia-the-viper

## Purpose

Julia the Viper: Harvard Architecture language for security-critical applications. Named after mathematician Julia Robinson and playful "adder" (snake + addition), JtV makes code injection grammatically impossible by separating computation into two distinct channels: Control Language (Turing-complete) and Data Language (total, addition-only).

## Module Map

```
julia-the-viper/
├── src/
│   ├── control_language/      # Turing-complete control channel
│   ├── data_language/         # Provably-halting data channel
│   ├── type_checker.rs        # Dual-channel type checking
│   ├── compiler.rs            # Code generation
│   └── runtime.rs             # Safe execution environment
├── test/
│   └── ... (security-critical tests)
├── examples/
│   └── ... (cryptography, sandboxing demos)
├── README.adoc                # Language specification
└── Cargo.toml                 # Rust package
```

## Data Flow

```
[JtV Source Code]
     ↙               ↘
[Control Language]  [Data Language]
(Turing-complete)   (Halting, addition-only)
     ↖               ↙
[Type Checker: Verify Channel Separation]
     ↓
[Compiled Binary: Injection-Proof]
```

## Key Invariants

- Control and Data channels are syntactically and semantically separate
- Data channel provably halts (no infinite loops)
- Code injection grammatically impossible at language level
- Designed for cryptography, sandboxing, security-critical systems
