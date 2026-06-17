# TEST-NEEDS.md — julia-the-viper

<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

## CRG Grade: B — Updated 2026-04-04

## Current Test State

| Suite | File | Count | Category |
|-------|------|-------|----------|
| Inline unit tests | `crates/jtv-core/src/*.rs` | 84 | UT |
| Pest rule tests | `tests/pest_rule_tests.rs` | 137 | UT/P2P |
| Parser integration | `tests/parser_tests.rs` | 22 | P2P |
| Property-based | `tests/property_tests.rs` | 41 | PBT |
| Harvard boundary | `tests/harvard_boundary_tests.rs` | 35 | P2P/SEC/SAF |
| Contract/invariant | `tests/contract_tests.rs` | 20 | CTR/LCY |
| Compatibility | `tests/compatibility_tests.rs` | 12 | CMP/VER |
| Mutation killers | `tests/mutation_killers.rs` | 156 | MUT targeting |
| Conformance | `tests/conformance_tests.rs` | 2 (31 valid + 21 invalid programs) | E2E |
| **Total passing** | | **509** | |

## Mutation Testing Results (cargo-mutants)

**Kill rate: 81.9%** on core files (number, typechecker, interpreter, pretty, ast, purity, reversible)
- 344 mutants caught
- 76 mutants missed
- 48 unviable
- 420 total non-unviable

## Proof Files

| Prover | Location | Files | Size | Status |
|--------|----------|-------|------|--------|
| Lean 4 | `jtv_proofs/` | 6 | ~80K | Compiles, no sorry |
| Idris2 | `src/abi/Types.idr` | 1 | 4.7K | Compiles, no believe_me |

## Benchmarks

| Suite | Location | Benchmarks |
|-------|----------|------------|
| Parser | `benches/parser_bench.rs` | LOC/sec at 50-1000 LOC, Harvard split |
| Interpreter | `benches/interpreter_bench.rs` | fibonacci, factorial, loops, pure calls |

## Fuzz Targets

| Target | Location | Input limit |
|--------|----------|-------------|
| fuzz_parser | `fuzz/fuzz_targets/fuzz_parser.rs` | 50K |
| fuzz_main | `fuzz/fuzz_targets/fuzz_main.rs` | 100K |

## Taxonomy Coverage Matrix

| Category | Abbrev | Status | Evidence |
|----------|--------|--------|----------|
| Unit | UT | PASS | 84 inline + 137 pest rules |
| Point-to-Point | P2P | PASS | 22 parser + 35 Harvard boundary |
| End-to-End | E2E | PARTIAL | 2 conformance failures (string/range syntax) |
| Build | BLD | PASS | CI `cargo check`, `just smoke` |
| Execution & Runtime | EXE | PASS | Interpreter tests, conformance valid programs |
| Reflexive | REF | PASS | `just doctor` (tools, structure, proofs, security, paths) |
| Lifecycle | LCY | PASS | Reversible interpreter lifecycle in contract_tests |
| Smoke | SMK | PASS | `just smoke` (<30s, 4 checks) |
| Property-Based | PBT | PASS | 41 proptest properties (7 number systems, reversibility) |
| Mutation | MUT | MISSING | Need cargo-mutants |
| Fuzz | FUZ | PASS | 2 libfuzzer targets |
| Contract/Invariant | CTR | PASS | 20 contract tests (addition-only, purity, reversibility) |
| Regression | REG | PARTIAL | Tests cover regressions, no dedicated directory yet |
| Chaos/Resilience | CHS | N/A | Not a distributed system |
| Compatibility | CMP | MISSING | No version matrix tests |
| Proof Regression | PRF | PASS | CI workflow `proof-regression.yml` |

## Aspect Coverage

| Aspect | Status | Evidence |
|--------|--------|----------|
| Security (SEC) | PASS | 35 Harvard boundary tests, fuzz targets, Lean security proof |
| Safety (SAF) | PASS | Addition-only contract, purity lattice, no dangerous patterns |
| Performance (PER) | PASS | 2 benchmark suites (parser + interpreter) |
| Functionality (FUN) | PASS | 339 passing tests, 51 conformance programs |
| Dependability (DEP) | PARTIAL | Overflow safety tested, no crash recovery tests |
| Interoperability (IOP) | PARTIAL | WASM tests exist, no FFI integration tests |
| Maintainability (MNT) | PASS | Deep annotation, `just doctor`, modular crates |
| Reproducibility (RPR) | PARTIAL | flake.nix exists, Lean proofs reproducible |
| Portability (PRT) | PARTIAL | CI on ubuntu-latest only |
| Usability (USA) | PARTIAL | LSP exists, no usability tests |
| Accessibility (ACC) | N/A | CLI/library, not UI |
| Privacy (PRI) | N/A | No user data handling |
| Observability (OBS) | PARTIAL | Debug crate exists, no structured logging tests |
| Versability (VER) | MISSING | No version compatibility matrix |

## What's Done (this session)

- [x] Property-based tests: 41 proptest properties across all 7 number systems
- [x] Harvard boundary P2P tests: 35 tests (data/control separation, injection impossibility)
- [x] Contract/invariant tests: 20 tests (addition-only, purity lattice, reversibility identity)
- [x] Smoke test: `just smoke` (build + parse + test + clippy in <30s)
- [x] Reflexive diagnostic: `just doctor` (tools, structure, proofs, security, test count)
- [x] Proof regression CI: `.github/workflows/proof-regression.yml` (Lean 4 + Idris2)
- [x] Clippy fix: removed unused enumerate in recovery.rs

## Still Missing (for CRG B)

- [ ] Mutation testing (cargo-mutants, target >80% kill rate)
- [ ] Version compatibility matrix (v0.1 data readable by future versions)
- [ ] Fix 2 pre-existing conformance failures (string literals in data_expr, range syntax)
- [ ] Zig FFI integration tests
- [ ] Cross-platform CI matrix (macOS, Windows)
- [ ] Coverage reporting (tarpaulin or llvm-cov)

## Run Tests

```bash
# All tests
cargo test -p jtv-core

# By category
just smoke                    # SMK: fast sanity check
just test                     # UT+P2P+E2E: full suite
just bench                    # PER: benchmarks
just doctor                   # REF: self-diagnostic
cargo test --test property_tests        # PBT
cargo test --test harvard_boundary_tests # SEC/SAF
cargo test --test contract_tests        # CTR/LCY
cargo +nightly fuzz run fuzz_parser     # FUZ
```
