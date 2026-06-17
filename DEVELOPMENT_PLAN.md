# Julia the Viper: Development Roadmap & Technical Decisions

**SPDX-License-Identifier: CC-BY-SA-4.0
**Version:** 1.0
**Date:** 2025

This document captures the strategic technical decisions and development roadmap for bringing JtV to production.

---

## 1. Strategic Technical Decisions

### 1.1 Compilation Strategy

**Decision: Hybrid Approach**

```
Phase 1: Interpreter + WASM Codegen (Web-first)
Phase 2: Add native compilation via Cranelift
Phase 3: Optional LLVM backend for maximum optimization
```

**Rationale:**
- WASM enables the playground (killer demo) immediately
- Cranelift is Rust-native, faster compile times than LLVM
- Interpreter remains for REPL and quick iteration
- LLVM can be added later for production optimization

**Implementation:**
```
┌─────────────────────────────────────────────────────────┐
│                    JtV Compilation                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│   Source (.jtv)                                         │
│        │                                                 │
│        ▼                                                 │
│   ┌─────────┐                                           │
│   │ Parser  │ ──► AST                                   │
│   └─────────┘                                           │
│        │                                                 │
│        ▼                                                 │
│   ┌─────────────┐                                       │
│   │ Type Check  │ ──► Typed AST                         │
│   │ Purity Check│                                       │
│   └─────────────┘                                       │
│        │                                                 │
│        ├──────────────┬──────────────┐                  │
│        ▼              ▼              ▼                  │
│   ┌─────────┐   ┌──────────┐   ┌──────────┐            │
│   │ Interp  │   │ WASM Gen │   │Cranelift │            │
│   │ (REPL)  │   │ (Web)    │   │ (Native) │            │
│   └─────────┘   └──────────┘   └──────────┘            │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

### 1.2 Module System Design

**Decision: Rust-style with URL-based packages**

```jtv
// Local module imports (Rust-style)
use math::sin
use math::{cos, tan}
use collections::*

// External package imports
use "github.com/org/package"::module
use "jtv.dev/std/crypto"::sha256

// Aliasing
use math::trigonometry as trig
```

**Package Naming: URL-based (Go-style)**
```
jtv.dev/std/prelude          # Standard library
github.com/user/package      # GitHub packages
gitlab.com/org/package       # GitLab packages
```

**Rationale:**
- Rust syntax is clean and explicit
- URL-based names are globally unique, no registry collision
- Works naturally with git repositories
- Familiar to Rust/Go developers

---

### 1.3 Standard Library Scope

**Decision: Minimal Core + Ecosystem Packages**

**Tier 1: Core (ships with compiler)**
```
std/prelude      # Auto-imported basics (already written)
std/result       # Error handling (already written)
std/option       # Optional values
```

**Tier 2: Standard (ships separately, official)**
```
std/math         # safe_math (already written)
std/collections  # Data structures (already written)
std/io           # File I/O
std/time         # Date/time
std/json         # JSON parsing
std/text         # String utilities
```

**Tier 3: Extended (official but optional)**
```
std/net          # HTTP, TCP, WebSocket
std/crypto       # Hashing, encryption (vetted)
std/regex        # Pattern matching
std/sql          # Database connectivity
```

**Tier 4: Community**
```
community/*      # Third-party packages
```

**Rationale:**
- Small core = fast compilation
- Batteries-included optional
- Security-sensitive (crypto) is official but audited
- Encourages ecosystem growth

---

### 1.4 Target Platforms

**Decision: Web-First, CLI-Second**

| Priority | Platform | Target | Use Case |
|----------|----------|--------|----------|
| P0 | Web Browser | WASM | Playground, embedded apps |
| P0 | CLI | Native | Developer tools, scripts |
| P1 | Server | Native/Container | Backend services |
| P2 | Mobile | Tauri 2.0 | Apps (if needed) |
| P3 | Embedded | Cranelift | IoT (future) |

**Rationale:**
- Playground is the killer demo (requires WASM)
- CLI tools are the developer workflow
- Server-side enables production use
- Mobile via Tauri per CLAUDE.md requirements

---

### 1.5 Package Registry

**Decision: Self-hosted, centralized, Deno-powered**

**Registry: jtv.dev**
```
https://jtv.dev/packages           # Web UI
https://api.jtv.dev/v1/packages    # API
https://jtv.dev/docs               # Documentation
```

**Architecture:**
```
┌─────────────────────────────────────────────────────┐
│                  jtv.dev Registry                    │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────────┐  ┌──────────────┐                │
│  │   Deno API   │  │  PostgreSQL  │                │
│  │   Server     │◄─┤  Database    │                │
│  └──────────────┘  └──────────────┘                │
│         │                                           │
│         ▼                                           │
│  ┌──────────────┐  ┌──────────────┐                │
│  │   S3/R2      │  │   CDN        │                │
│  │   Storage    │──┤   (bunny.net)│                │
│  └──────────────┘  └──────────────┘                │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Features:**
- Package publishing via CLI (`jtv publish`)
- Version management (SemVer)
- Dependency resolution
- Security scanning
- Documentation hosting
- Download statistics

---

### 1.6 Development Priority Order

**Decision: Web-First (Option A)**

```
Phase 1: Foundation (Months 1-2)
├── Fix module system
├── Fix function dispatch
├── Integrate type checking
└── Integrate purity enforcement

Phase 2: Web Platform (Months 2-3)
├── WASM code generation
├── Playground MVP
├── Router Visualization
└── Monaco editor integration

Phase 3: Developer Tools (Months 3-4)
├── LSP server
├── VS Code extension (ReScript)
├── Debugger basics
└── Test runner

Phase 4: Ecosystem (Months 4-5)
├── Package manager CLI
├── Registry (jtv.dev)
├── Documentation site
└── Standard library completion

Phase 5: Production (Months 5-6)
├── Native compilation (Cranelift)
├── Performance optimization
├── Security audit
└── v1.0 release
```

---

### 1.7 Versioning Strategy

**Decision: SemVer with pre-release tags**

```
1.0.0-alpha.1    # Early development
1.0.0-alpha.2
1.0.0-beta.1     # Feature complete, testing
1.0.0-beta.2
1.0.0-rc.1       # Release candidate
1.0.0            # Stable release
1.0.1            # Patch (bug fixes)
1.1.0            # Minor (new features, backward compatible)
2.0.0            # Major (breaking changes, v2 features)
```

**Version Guarantees:**
- Patch: No breaking changes, bug fixes only
- Minor: New features, deprecations, no breaking changes
- Major: Breaking changes allowed, migration guide required

---

### 1.8 Formal Verification Integration

**Decision: CI-integrated, not runtime**

```yaml
# .github/workflows/proofs.yml
name: Formal Verification
on: [push, pull_request]
jobs:
  lean:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: leanprover/lean4-action@v1
      - run: cd jtv_proofs && lake build
      - run: cd jtv_proofs && lake test
```

**Policy:**
- All PRs must pass Lean proof compilation
- New language features require corresponding proofs
- Security properties must be mechanically verified
- Performance is not verified (too complex)

---

### 1.9 Backward Compatibility

**Decision: Tooling-assisted migration**

**v1 → v2 Migration:**
```bash
# Automatic migration tool
jtv migrate --from v1 --to v2 src/

# Reports:
# - Deprecated features used
# - Breaking changes detected
# - Automatic fixes applied
# - Manual fixes required
```

**Compatibility Policy:**
- v1.x: Fully backward compatible
- v2.0: Breaking changes, 6-month deprecation warnings
- Migration tool provided for all major versions
- LTS versions supported for 2 years

---

### 1.10 Governance Model

**Decision: RFC process with core team**

**Structure:**
```
Core Team (3-5 people)
├── Language design decisions
├── Final approval on RFCs
├── Release management
└── Security response

Contributors
├── Submit PRs
├── Propose RFCs
├── Review code
└── Write documentation

Community
├── Report issues
├── Request features
├── Provide feedback
└── Build ecosystem
```

**RFC Process:**
1. Open RFC issue with proposal
2. Community discussion (2 weeks minimum)
3. Core team review
4. Accept/Reject/Request changes
5. Implementation begins after acceptance

---

## 2. Complete Technology Stack

### 2.1 Core Language (Rust)

```toml
# Cargo.toml workspace
[workspace]
members = [
    "crates/jtv-core",        # Parser, AST, type system
    "crates/jtv-interp",      # Interpreter
    "crates/jtv-wasm",        # WASM codegen
    "crates/jtv-native",      # Cranelift codegen
    "crates/jtv-lsp",         # Language server
    "crates/jtv-cli",         # Command-line tools
    "crates/jtv-pkg",         # Package manager
    "crates/jtv-fmt",         # Formatter
    "crates/jtv-test",        # Test runner
]

[workspace.dependencies]
# Parser
pest = "2.7"
pest_derive = "2.7"

# Type system
im = "15"                     # Immutable data structures

# WASM
wasm-encoder = "0.41"
wasmparser = "0.121"

# Native compilation
cranelift = "0.104"
cranelift-module = "0.104"
cranelift-native = "0.104"

# LSP
tower-lsp = "0.20"
async-std = "1.12"

# CLI
clap = { version = "4", features = ["derive"] }
colored = "2"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Error handling
thiserror = "1"
miette = "7"                  # Beautiful diagnostics

# Testing
insta = "1"                   # Snapshot testing
criterion = "0.5"             # Benchmarks
```

### 2.2 Web Platform (ReScript + Deno)

```json
// rescript.json (playground)
{
  "name": "jtv-playground",
  "sources": ["src"],
  "package-specs": {
    "module": "esmodule",
    "in-source": true
  },
  "suffix": ".res.js",
  "bs-dependencies": [
    "@halcyon/rescript-tea",
    "@rescript/core"
  ]
}
```

**Framework: rescript-tea (The Elm Architecture for ReScript)**

The playground uses [`rescript-tea`](https://github.com/halcyon/rescript-tea) which implements
The Elm Architecture (TEA) / Model-View-Update (MVU) pattern. This provides:

- **Immutable state management** - Single source of truth
- **Unidirectional data flow** - Predictable updates
- **Declarative rendering** - Virtual DOM diffing
- **Natural fit for JtV** - Functional paradigm matches language philosophy

**Key Components to Build:**

| Component | Description | Location |
|-----------|-------------|----------|
| `cadre-tea-router` | Router Visualization component showing Control/Data channel separation | `playground/src/CadreTeaRouter.res` |
| `JtvEditor` | Monaco editor wrapper with JtV syntax highlighting | `playground/src/JtvEditor.res` |
| `NumberExplorer` | Interactive showcase of 7 number systems | `playground/src/NumberExplorer.res` |
| `WasmRunner` | WASM execution bridge | `playground/src/WasmRunner.res` |

**cadre-tea-router Architecture:**

```
┌─────────────────────────────────────────────────────────────┐
│                  cadre-tea-router                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐     ┌─────────────────┐               │
│  │   CONTROL       │     │     DATA        │               │
│  │   CHANNEL       │     │    CHANNEL      │               │
│  │   (Blue)        │     │    (Red)        │               │
│  │                 │     │                 │               │
│  │  Loops          │     │  Expressions    │               │
│  │  Conditionals   │     │  Calculations   │               │
│  │  I/O            │     │  Pure functions │               │
│  │  Assignments    │     │  7 number types │               │
│  │                 │     │                 │               │
│  └────────┬────────┘     └────────┬────────┘               │
│           │                       │                         │
│           └───────────┬───────────┘                         │
│                       │                                     │
│              ┌────────▼────────┐                           │
│              │  BRIDGE         │                           │
│              │  (Data → Ctrl)  │                           │
│              │  One-way only   │                           │
│              └─────────────────┘                           │
│                                                              │
│  Legend: Animated flow showing grammatical separation       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

```json
// deno.json (registry API)
{
  "name": "jtv-registry",
  "version": "1.0.0",
  "exports": "./src/main.ts",
  "tasks": {
    "dev": "deno run --watch src/main.ts",
    "start": "deno run --allow-net --allow-read src/main.ts"
  },
  "imports": {
    "oak": "jsr:@oak/oak@^14",
    "postgres": "jsr:@db/postgres@^0.19"
  }
}
```

### 2.3 VS Code Extension (ReScript)

```json
// package.json
{
  "name": "jtv-vscode",
  "displayName": "Julia the Viper",
  "publisher": "jtv",
  "engines": { "vscode": "^1.85.0" },
  "categories": ["Programming Languages"],
  "activationEvents": ["onLanguage:jtv"],
  "main": "./out/extension.res.js",
  "contributes": {
    "languages": [{
      "id": "jtv",
      "extensions": [".jtv"],
      "configuration": "./language-configuration.json"
    }],
    "grammars": [{
      "language": "jtv",
      "scopeName": "source.jtv",
      "path": "./syntaxes/jtv.tmLanguage.json"
    }]
  }
}
```

### 2.4 Build & CI

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --all
      - run: cargo test --all
      - run: cargo clippy --all -- -D warnings

  wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - run: cargo install wasm-pack
      - run: wasm-pack build crates/jtv-wasm

  lean:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: leanprover/lean4-action@v1
      - run: cd jtv_proofs && lake build

  rescript:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: denoland/setup-deno@v1
      - run: cd playground && deno task build
```

### 2.5 Container Runtime

**Decision: nerdctl (primary), podman (fallback)**

**NO DOCKER** - per project policy, use containerd-native tools:

```bash
# Primary: nerdctl (containerd-native, Docker-compatible CLI)
nerdctl build -t jtv-dev .
nerdctl run -it --rm jtv-dev

# Fallback: podman (daemonless, rootless)
podman build -t jtv-dev .
podman run -it --rm jtv-dev
```

**Rationale:**
- **nerdctl**: Native containerd integration, Docker CLI compatible, lighter weight
- **podman**: Daemonless, rootless by default, OCI-compliant
- Both are FOSS with no vendor lock-in
- CI uses GitHub Actions runners (no containers needed for most jobs)

**Container Use Cases:**
| Use Case | Tool | Notes |
|----------|------|-------|
| Local development | nerdctl/podman | Optional, for isolation |
| CI builds | GitHub Actions | Native runners preferred |
| Registry deployment | nerdctl | Production containers |
| Local testing | None | Prefer native `cargo test` |

**Containerfile (multi-stage):**
```dockerfile
# Build stage
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/jtv /usr/local/bin/
ENTRYPOINT ["jtv"]
```

---

## 3. Directory Structure

```
julia-the-viper/
├── crates/                      # Rust workspace
│   ├── jtv-core/               # Parser, AST, types
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parser/
│   │   │   ├── ast/
│   │   │   ├── types/
│   │   │   └── purity/
│   │   └── Cargo.toml
│   ├── jtv-interp/             # Interpreter
│   ├── jtv-wasm/               # WASM codegen
│   ├── jtv-native/             # Cranelift codegen
│   ├── jtv-lsp/                # Language server
│   ├── jtv-cli/                # CLI tools
│   ├── jtv-pkg/                # Package manager
│   ├── jtv-fmt/                # Formatter
│   └── jtv-test/               # Test runner
│
├── std/                         # Standard library
│   ├── prelude/
│   ├── math/
│   ├── collections/
│   ├── io/
│   ├── time/
│   └── json/
│
├── playground/                  # Web playground (ReScript + rescript-tea)
│   ├── src/
│   │   ├── Main.res             # Entry point, TEA app setup
│   │   ├── Model.res            # Application state
│   │   ├── Update.res           # Message handlers
│   │   ├── View.res             # Main view
│   │   ├── CadreTeaRouter.res   # Router Visualization (Control/Data channels)
│   │   ├── JtvEditor.res        # Monaco editor integration
│   │   ├── NumberExplorer.res   # 7 number systems showcase
│   │   ├── WasmRunner.res       # WASM execution bridge
│   │   └── Subscriptions.res    # Event subscriptions
│   ├── rescript.json
│   ├── deno.json                # Deno for dev server
│   └── index.html
│
├── registry/                    # Package registry (Deno)
│   ├── src/
│   │   ├── main.ts
│   │   ├── api/
│   │   └── db/
│   └── deno.json
│
├── editors/                     # IDE extensions
│   ├── vscode/                 # VS Code (ReScript)
│   ├── neovim/                 # Neovim (Lua + TreeSitter)
│   └── zed/                    # Zed (Rust)
│
├── docs/                        # Documentation
│   ├── book/                   # mdBook source
│   ├── api/                    # Generated API docs
│   └── rfcs/                   # RFC documents
│
├── jtv_proofs/                  # Lean 4 proofs
│   ├── JtvCore.lean
│   ├── JtvTypes.lean
│   ├── JtvSecurity.lean
│   └── lakefile.lean
│
├── examples/                    # Example programs
├── tests/                       # Integration tests
├── benches/                     # Benchmarks
│
├── Cargo.toml                   # Workspace root
├── Justfile                     # Build commands
├── flake.nix                    # Nix packaging
└── guix.scm                     # Guix packaging
```

---

## 4. Development Milestones

### Milestone 1: Foundation (Weeks 1-4)

**Goal:** Working language core

| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Restructure to workspace | - | 3 days | - |
| Fix module imports | - | 1 week | Workspace |
| Fix function dispatch | - | 1 week | Module imports |
| Integrate type checker | - | 1 week | Function dispatch |
| Integrate purity checker | - | 1 week | Type checker |
| Add error recovery | - | 3 days | All above |

**Deliverable:** `jtv run` works for all examples

### Milestone 2: Web Platform (Weeks 5-8)

**Goal:** Playground MVP

| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| WASM code generation | - | 2 weeks | M1 complete |
| Playground scaffold (ReScript) | - | 1 week | - |
| Monaco editor integration | - | 3 days | Playground |
| Router Visualization | - | 1 week | WASM |
| Deploy to jtv.dev | - | 2 days | All above |

**Deliverable:** Interactive playground at jtv.dev

### Milestone 3: Developer Tools (Weeks 9-12)

**Goal:** IDE support

| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| LSP server (tower-lsp) | - | 2 weeks | M1 complete |
| VS Code extension (ReScript) | - | 1 week | LSP |
| Diagnostics/errors | - | 3 days | LSP |
| Hover/completion | - | 1 week | LSP |
| Test runner (`jtv test`) | - | 1 week | M1 complete |

**Deliverable:** VS Code extension with IntelliSense

### Milestone 4: Ecosystem (Weeks 13-16)

**Goal:** Package ecosystem

| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Package manager CLI | - | 2 weeks | M1 complete |
| Registry API (Deno) | - | 2 weeks | - |
| Registry web UI | - | 1 week | API |
| Publish std packages | - | 3 days | Registry |
| Documentation site | - | 1 week | - |

**Deliverable:** `jtv install` works, jtv.dev/packages live

### Milestone 5: Production (Weeks 17-20)

**Goal:** v1.0 release

| Task | Owner | Duration | Dependencies |
|------|-------|----------|--------------|
| Cranelift backend | - | 2 weeks | M1 complete |
| Performance optimization | - | 1 week | Cranelift |
| Security audit | - | 1 week | All |
| Documentation complete | - | 1 week | All |
| Release automation | - | 3 days | All |

**Deliverable:** JtV v1.0.0 released

---

## 5. Success Metrics

### Technical Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Parse speed | >100k lines/sec | Benchmark |
| Type check speed | >50k lines/sec | Benchmark |
| WASM size | <100KB (core) | Build output |
| Playground load | <2 seconds | Lighthouse |
| LSP response | <100ms | Profiling |
| Test coverage | >80% | Coverage tool |

### Adoption Metrics

| Metric | Target (6 months) | Target (1 year) |
|--------|-------------------|-----------------|
| GitHub stars | 1,000 | 5,000 |
| Packages published | 50 | 500 |
| Monthly downloads | 1,000 | 10,000 |
| Discord members | 100 | 1,000 |
| Contributors | 10 | 50 |

---

## 6. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| WASM codegen complexity | Medium | High | Start simple, iterate |
| LSP development time | Medium | Medium | Use tower-lsp, copy patterns |
| Adoption challenges | High | High | Focus on playground demo |
| Security vulnerabilities | Low | Critical | Audit, fuzzing, Lean proofs |
| Maintainer burnout | Medium | High | Build contributor community |

---

## 7. Open Questions (To Resolve During Development)

1. **Error message format:** Rust-style? Elm-style? Custom?
2. **REPL persistence:** Save session history? Load files?
3. **Debug symbols:** DWARF? Custom format?
4. **Incremental compilation:** Worth the complexity for v1?
5. **Hot reloading:** For playground? For development?

---

## Appendix: Quick Reference

### Commands After v1.0

```bash
# Installation
curl -fsSL https://jtv.dev/install.sh | sh

# Project management
jtv new my-project          # Create new project
jtv init                    # Initialize in existing directory
jtv build                   # Compile project
jtv run                     # Run main.jtv
jtv test                    # Run tests
jtv bench                   # Run benchmarks

# Package management
jtv install <package>       # Add dependency
jtv remove <package>        # Remove dependency
jtv update                  # Update dependencies
jtv publish                 # Publish to registry

# Development tools
jtv fmt                     # Format code
jtv lint                    # Lint code
jtv check                   # Type check without running
jtv doc                     # Generate documentation
jtv repl                    # Interactive shell

# Targets
jtv build --target wasm     # Compile to WASM
jtv build --target native   # Compile to native binary
jtv build --release         # Optimized build
```

### Project Structure (After `jtv new`)

```
my-project/
├── src/
│   └── main.jtv
├── tests/
│   └── main_test.jtv
├── jtv.toml                # Project manifest
└── jtv.lock                # Dependency lock
```

### jtv.toml Format

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2025"

[dependencies]
"jtv.dev/std/json" = "1.0"
"github.com/user/lib" = "0.5"

[dev-dependencies]
"jtv.dev/std/test" = "1.0"

[build]
target = "wasm"             # or "native"
optimize = true
```
