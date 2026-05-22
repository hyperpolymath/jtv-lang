# SPDX-License-Identifier: MPL-2.0
# Justfile - Julia the Viper build system

# Default: show all available commands
import? "contractile.just"

default:
    @just --list

# Build all packages in workspace
build:
    @echo "🔨 Building workspace..."
    cargo build --workspace

# Build with release optimizations
build-release:
    @echo "🚀 Building release..."
    cargo build --workspace --release

# Build WASM target for web playground
build-wasm:
    @echo "🌐 Building WASM..."
    cd crates/jtv-core && wasm-pack build --target web --out-dir ../../pkg
    @echo "✅ WASM build complete: pkg/"

# Run all tests
test:
    @echo "🧪 Running tests..."
    cargo test --workspace

# Run tests with output
test-verbose:
    @echo "🧪 Running tests (verbose)..."
    cargo test --workspace -- --nocapture

# Run a specific test
test-one TEST:
    @echo "🧪 Running test: {{TEST}}"
    cargo test --workspace {{TEST}} -- --nocapture

# Run all lints (clippy)
lint:
    @echo "🔍 Running clippy..."
    cargo clippy --workspace -- -D warnings

# Format all code
fmt:
    @echo "✨ Formatting code..."
    cargo fmt --all

# Check formatting without changes
fmt-check:
    @echo "👀 Checking formatting..."
    cargo fmt --all -- --check

# Run all checks (lint + test + fmt-check)
check: lint test fmt-check
    @echo "✅ All checks passed!"

# Clean build artifacts
clean:
    @echo "🧹 Cleaning..."
    cargo clean
    rm -rf pkg/
    rm -rf target/

# Install CLI binary
install:
    @echo "📦 Installing jtv CLI..."
    cargo install --path crates/jtv-cli

# Run the REPL
repl:
    @echo "🐍 Starting JtV REPL..."
    cargo run --bin jtv repl

# Run a JtV file
run FILE:
    @echo "▶️  Running {{FILE}}..."
    cargo run --bin jtv run {{FILE}}

# Run a file with trace enabled
run-trace FILE:
    @echo "🔬 Running {{FILE}} with trace..."
    cargo run --bin jtv run --trace {{FILE}}

# Parse a file and show AST
parse FILE:
    @echo "🌳 Parsing {{FILE}}..."
    cargo run --bin jtv parse {{FILE}}

# Check a file for errors
check-file FILE:
    @echo "🔍 Checking {{FILE}}..."
    cargo run --bin jtv check {{FILE}}

# Run all example programs
run-examples:
    @echo "🎯 Running all examples..."
    @for file in examples/basic/*.jtv; do \
        echo "\n▶️  $$file"; \
        cargo run --quiet --bin jtv run "$$file" || exit 1; \
    done
    @for file in examples/advanced/*.jtv; do \
        echo "\n▶️  $$file"; \
        cargo run --quiet --bin jtv run "$$file" || exit 1; \
    done
    @echo "\n✅ All examples ran successfully!"

# Run benchmarks
bench:
    @echo "⚡ Running benchmarks..."
    cargo bench --workspace

# Check RSR compliance
rsr:
    @echo "📋 Checking RSR compliance..."
    cargo run --bin jtv rsr-check

# Generate documentation
doc:
    @echo "📚 Generating documentation..."
    cargo doc --workspace --no-deps --open

# Update dependencies
update:
    @echo "⬆️  Updating dependencies..."
    cargo update

# Audit dependencies for security issues
audit:
    @echo "🔒 Auditing dependencies..."
    cargo audit

# Prepare a release (requires VERSION argument)
release VERSION:
    @echo "🎉 Preparing release {{VERSION}}..."
    @echo "1. Updating Cargo.toml versions..."
    @sed -i 's/^version = .*/version = "{{VERSION}}"/' crates/jtv-core/Cargo.toml
    @sed -i 's/^version = .*/version = "{{VERSION}}"/' crates/jtv-cli/Cargo.toml
    @sed -i 's/^version = .*/version = "{{VERSION}}"/' Cargo.toml
    @echo "2. Running checks..."
    just check
    @echo "3. Building release..."
    just build-release
    @echo "4. Building WASM..."
    just build-wasm
    @echo "5. Running examples..."
    just run-examples
    @echo "✅ Release {{VERSION}} ready! Now:"
    @echo "   git add -A"
    @echo "   git commit -m 'chore: release v{{VERSION}}'"
    @echo "   git tag v{{VERSION}}"
    @echo "   git push && git push --tags"

# Watch for changes and run tests
watch:
    @echo "👀 Watching for changes..."
    cargo watch -x "test --workspace"

# Watch and run a specific example
watch-run FILE:
    @echo "👀 Watching {{FILE}}..."
    cargo watch -x "run --bin jtv run {{FILE}}"

# Validate project structure (Nix build)
validate:
    @echo "🔧 Validating with Nix..."
    nix flake check

# Enter Nix development shell
dev:
    @echo "🐚 Entering development shell..."
    nix develop

# CI workflow simulation
ci: fmt-check lint test build-wasm run-examples rsr
    @echo "✅ CI checks complete!"

# Show project statistics
stats:
    @echo "📊 Project Statistics:"
    @echo ""
    @echo "Lines of Rust code:"
    @find crates -name "*.rs" -exec wc -l {} + | tail -1
    @echo ""
    @echo "Example programs:"
    @find examples -name "*.jtv" | wc -l
    @echo ""
    @echo "Test count:"
    @grep -r "#\[test\]" crates/ | wc -l
    @echo ""
    @echo "Crates:"
    @ls -1 crates/ | wc -l

# Quick iteration: fmt + check
quick: fmt check
    @echo "⚡ Quick checks done!"

# === PWA Development ===

# Serve PWA locally
serve-pwa: build-wasm
    @echo "🌐 Starting PWA server..."
    cd web && deno task serve

# Serve PWA with hot reload
dev-pwa: build-wasm
    @echo "🔥 Starting PWA dev server with hot reload..."
    cd web && deno task dev

# Build PWA for production
build-pwa: build-wasm
    @echo "📦 Building PWA for production..."
    @echo "✅ WASM compiled to pkg/"
    @echo "✅ PWA files in web/"
    @echo "📝 Ready to deploy!"

# Smoke test: fast sanity check (<30 seconds). Taxonomy: SMK
smoke:
    @echo "Running smoke test..."
    @echo "  [1/4] Cargo check (jtv-core)..."
    @cargo check --quiet -p jtv-core 2>&1
    @echo "  [OK] Build compiles"
    @echo "  [2/4] Parse conformance file..."
    @cargo run --quiet --bin jtv parse conformance/valid/data_expressions.jtv 2>&1 > /dev/null
    @echo "  [OK] Parser works"
    @echo "  [3/4] Run unit tests (quick)..."
    @cargo test --quiet -p jtv-core --lib 2>&1 > /dev/null
    @echo "  [OK] Unit tests pass"
    @echo "  [4/4] Clippy (jtv-core, no warnings)..."
    @cargo clippy --quiet -p jtv-core -- -D warnings 2>&1 > /dev/null
    @echo "  [OK] No clippy warnings"
    @echo "Smoke test passed."

# Run panic-attacker pre-commit scan
assail:
    @command -v panic-attack >/dev/null 2>&1 && panic-attack assail . || echo "panic-attack not found — install from https://github.com/hyperpolymath/panic-attacker"

# Self-diagnostic — checks dependencies, permissions, paths, proofs. Taxonomy: REF
doctor:
    @echo "Running diagnostics for julia-the-viper..."
    @FAIL=0; \
    echo "=== Required Tools ==="; \
    command -v just >/dev/null 2>&1 && echo "  [OK] just" || { echo "  [FAIL] just not found"; FAIL=1; }; \
    command -v cargo >/dev/null 2>&1 && echo "  [OK] cargo ($(cargo --version 2>/dev/null | head -c 20))" || { echo "  [FAIL] cargo not found"; FAIL=1; }; \
    command -v git >/dev/null 2>&1 && echo "  [OK] git" || { echo "  [FAIL] git not found"; FAIL=1; }; \
    echo "=== Project Structure ==="; \
    test -f Cargo.toml && echo "  [OK] Cargo.toml" || { echo "  [FAIL] Cargo.toml missing"; FAIL=1; }; \
    test -f Justfile && echo "  [OK] Justfile" || { echo "  [FAIL] Justfile missing"; FAIL=1; }; \
    test -f LICENSE && echo "  [OK] LICENSE" || { echo "  [FAIL] LICENSE missing"; FAIL=1; }; \
    test -d crates/jtv-core/src && echo "  [OK] crates/jtv-core/src/" || { echo "  [FAIL] core crate missing"; FAIL=1; }; \
    test -f crates/jtv-core/src/grammar.pest && echo "  [OK] grammar.pest" || { echo "  [FAIL] grammar.pest missing"; FAIL=1; }; \
    echo "=== Formal Proofs ==="; \
    test -d jtv_proofs && echo "  [OK] jtv_proofs/ (Lean 4)" || echo "  [WARN] jtv_proofs/ missing"; \
    test -f src/abi/Types.idr && echo "  [OK] src/abi/Types.idr (Idris2)" || echo "  [WARN] src/abi/Types.idr missing"; \
    echo "=== Conformance Suite ==="; \
    VALID=$(ls conformance/valid/*.jtv 2>/dev/null | wc -l); \
    INVALID=$(ls conformance/invalid/*.jtv 2>/dev/null | wc -l); \
    echo "  [INFO] $VALID valid programs, $INVALID invalid programs"; \
    echo "=== Security Checks ==="; \
    grep -rn 'believe_me\|assert_total\|sorry\|Admitted\|unsafeCoerce' --include='*.rs' --include='*.idr' --include='*.lean' . 2>/dev/null | grep -v target/ | head -5 && { echo "  [WARN] Dangerous patterns found"; } || echo "  [OK] No dangerous patterns"; \
    echo "=== Hardcoded Paths ==="; \
    grep -rn '\$HOME\|\$ECLIPSE_DIR\|/home/' --include='*.rs' --include='*.ex' --include='*.sh' . 2>/dev/null | grep -v target/ | head -5 && echo "  [WARN] Hardcoded paths found" || echo "  [OK] No hardcoded paths"; \
    echo "=== Test Count ==="; \
    TESTS=$(grep -r '#\[test\]' crates/ --include='*.rs' 2>/dev/null | grep -v target/ | wc -l); \
    echo "  [INFO] $TESTS tests found"; \
    if [ $FAIL -gt 0 ]; then echo "DIAGNOSTICS: FAILED ($FAIL issues)"; exit 1; else echo "DIAGNOSTICS: PASSED"; fi

# Auto-repair common issues
heal:
    @echo "Attempting auto-repair for julia-the-viper..."
    @echo "Fixing permissions..."
    @find . -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true
    @echo "Cleaning stale caches..."
    @rm -rf .cache/stale 2>/dev/null || true
    @echo "Repair complete."

# Guided tour of key features
tour:
    @echo "=== julia-the-viper Tour ==="
    @echo ""
    @echo "1. Project structure:"
    @ls -la
    @echo ""
    @echo "2. Available commands: just --list"
    @echo ""
    @echo "3. Read README.adoc for full overview"
    @echo "4. Read EXPLAINME.adoc for architecture decisions"
    @echo "5. Run 'just doctor' to check your setup"
    @echo ""
    @echo "Tour complete! Try 'just --list' to see all available commands."

# Open feedback channel with diagnostic context
help-me:
    @echo "=== julia-the-viper Help ==="
    @echo "Platform: $(uname -s) $(uname -m)"
    @echo "Shell: $SHELL"
    @echo ""
    @echo "To report an issue:"
    @echo "  https://github.com/hyperpolymath/julia-the-viper/issues/new"
    @echo ""
    @echo "Include the output of 'just doctor' in your report."


# Print the current CRG grade (reads from READINESS.md '**Current Grade:** X' line)
crg-grade:
    @grade=$$(grep -oP '(?<=\*\*Current Grade:\*\* )[A-FX]' READINESS.md 2>/dev/null | head -1); \
    [ -z "$$grade" ] && grade="X"; \
    echo "$$grade"

# Generate a shields.io badge markdown for the current CRG grade
# Looks for '**Current Grade:** X' in READINESS.md; falls back to X
crg-badge:
    @grade=$$(grep -oP '(?<=\*\*Current Grade:\*\* )[A-FX]' READINESS.md 2>/dev/null | head -1); \
    [ -z "$$grade" ] && grade="X"; \
    case "$$grade" in \
      A) color="brightgreen" ;; B) color="green" ;; C) color="yellow" ;; \
      D) color="orange" ;; E) color="red" ;; F) color="critical" ;; \
      *) color="lightgrey" ;; esac; \
    echo "[![CRG $$grade](https://img.shields.io/badge/CRG-$$grade-$$color?style=flat-square)](https://github.com/hyperpolymath/standards/tree/main/component-readiness-grades)"
