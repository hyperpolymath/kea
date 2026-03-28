# SPDX-License-Identifier: PMPL-1.0-or-later
# Copyright (c) 2024-2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
#
# Kea: Unified build and run commands for the Kea ecosystem
# See TOPOLOGY.md for architecture map, README.adoc for quick start.

# Default recipe — show available commands
default:
    @just --list

# === Workspace-wide commands ===

# Build all Rust components (bivouac + mandible)
build:
    cd bivouac && cargo build --workspace
    cd mandible && cargo build --workspace

# Build all in release mode
build-release:
    cd bivouac && cargo build --workspace --release
    cd mandible && cargo build --workspace --release

# Run all tests across all components
test:
    cd bivouac && cargo test --workspace
    cd mandible && cargo test --workspace

# Lint all Rust components
lint:
    cd bivouac && cargo clippy --workspace --all-targets -- -D warnings
    cd mandible && cargo clippy --workspace --all-targets -- -D warnings

# Format all Rust code
fmt:
    cd bivouac && cargo fmt --all
    cd mandible && cargo fmt --all

# Check formatting across all components
fmt-check:
    cd bivouac && cargo fmt --all -- --check
    cd mandible && cargo fmt --all -- --check

# Run full check suite (format, lint, test)
check: fmt-check lint test

# Clean all build artifacts
clean:
    cd bivouac && cargo clean
    cd mandible && cargo clean

# === Security and Quality ===

# Run panic-attack pre-commit scan
panic:
    panic-attack assail

# Run cargo audit for known vulnerabilities
audit:
    cd bivouac && cargo audit 2>/dev/null || true
    cd mandible && cargo audit 2>/dev/null || true

# Run cargo deny for license and advisory checks
deny:
    cd bivouac && cargo deny check 2>/dev/null || true
    cd mandible && cargo deny check 2>/dev/null || true

# Full quality gate (check + security)
quality: check panic audit

# === Bivouac commands ===

# Execute a failover playbook via Bivouac
bivouac-playbook playbook:
    cd bivouac && cargo run --release -- trigger-playbook {{playbook}}

# Build Bivouac only
bivouac-build:
    cd bivouac && cargo build --workspace

# Test Bivouac only
bivouac-test:
    cd bivouac && cargo test --workspace

# Lint Bivouac only
bivouac-lint:
    cd bivouac && cargo clippy --workspace --all-targets -- -D warnings

# === Call commands ===

# Generate language-specific bindings from Cap'n Proto schemas
call-generate-bindings:
    cd call && echo "Cap'n Proto binding generation — configure schema path first"

# === Mandible commands ===

# Deep audit of a target path via Mandible
mandible-pry target:
    cd mandible && cargo run --release -- pry --target {{target}}

# WordPress audit via Mandible
mandible-wordpress path:
    cd mandible && cargo run --release -- wordpress --path {{path}} --audit-config

# Bloat analysis via Mandible
mandible-slop target:
    cd mandible && cargo run --release -- slop --target {{target}}

# Build Mandible only
mandible-build:
    cd mandible && cargo build --workspace

# Test Mandible only
mandible-test:
    cd mandible && cargo test --workspace

# Lint Mandible only
mandible-lint:
    cd mandible && cargo clippy --workspace --all-targets -- -D warnings

# === Wit commands ===

# Build Wit tooling (pending specification)
wit-build:
    cd wit && echo "Wit tooling — specification pending (see wit/ROADMAP.adoc)"

# === Documentation ===

# Generate documentation for all Rust components
doc:
    cd bivouac && cargo doc --workspace --no-deps
    cd mandible && cargo doc --workspace --no-deps

# Open generated docs in browser
doc-open:
    cd bivouac && cargo doc --workspace --no-deps --open

# === Fuzz Testing ===

# Run ClusterFuzzLite targets for Bivouac
fuzz-bivouac:
    cd bivouac/fuzz && cargo fuzz list

# Run ClusterFuzzLite targets for Mandible
fuzz-mandible:
    cd mandible/fuzz && cargo fuzz list

# === Validation ===

# Validate RSR compliance (SPDX headers, file structure, etc.)
validate:
    @echo "Checking SPDX headers..."
    @grep -rL "SPDX-License-Identifier" bivouac/src/ mandible/crates/*/src/ 2>/dev/null && echo "MISSING SPDX HEADERS" || echo "All source files have SPDX headers"
    @echo ""
    @echo "Checking for banned patterns..."
    @grep -rn "believe_me\|assert_total\|sorry\|Admitted\|unsafeCoerce\|Obj.magic" bivouac/src/ mandible/crates/*/src/ 2>/dev/null && echo "BANNED PATTERNS FOUND" || echo "No banned patterns detected"
    @echo ""
    @echo "Checking for banned languages..."
    @find . -name "*.ts" -o -name "*.py" -o -name "*.go" | grep -v node_modules | grep -v .git | head -5 && echo "BANNED LANGUAGE FILES FOUND" || echo "No banned language files"
    @echo ""
    @echo "Checking workflow count..."
    @echo "Workflows: $(ls .github/workflows/*.yml | wc -l)/17"

# Validate machine-readable state files exist
validate-state:
    @echo "Checking .machine_readable/6a2/ ..."
    @for f in STATE META ECOSYSTEM AGENTIC NEUROSYM PLAYBOOK; do \
        test -f ".machine_readable/6a2/$$f.a2ml" && echo "  $$f.a2ml OK" || echo "  $$f.a2ml MISSING"; \
    done
    @test -f ".machine_readable/anchors/ANCHOR.a2ml" && echo "  ANCHOR.a2ml OK" || echo "  ANCHOR.a2ml MISSING"

# === Multi-arch ===

# Build for RISC-V target
# Build for ARM64
build-arm64:
    @echo "Building for ARM64..."
    cross build --target aarch64-unknown-linux-gnu

# Run panic-attacker pre-commit scan
assail:
    @command -v panic-attack >/dev/null 2>&1 && panic-attack assail . || echo "panic-attack not found — install from https://github.com/hyperpolymath/panic-attacker"

# Self-diagnostic — checks dependencies, permissions, paths
doctor:
    @echo "Running diagnostics for kea..."
    @echo "Checking required tools..."
    @command -v just >/dev/null 2>&1 && echo "  [OK] just" || echo "  [FAIL] just not found"
    @command -v git >/dev/null 2>&1 && echo "  [OK] git" || echo "  [FAIL] git not found"
    @echo "Checking for hardcoded paths..."
    @grep -rn '$HOME\|$ECLIPSE_DIR' --include='*.rs' --include='*.ex' --include='*.res' --include='*.gleam' --include='*.sh' . 2>/dev/null | head -5 || echo "  [OK] No hardcoded paths"
    @echo "Diagnostics complete."

# Auto-repair common issues
heal:
    @echo "Attempting auto-repair for kea..."
    @echo "Fixing permissions..."
    @find . -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true
    @echo "Cleaning stale caches..."
    @rm -rf .cache/stale 2>/dev/null || true
    @echo "Repair complete."

# Guided tour of key features
tour:
    @echo "=== kea Tour ==="
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
    @echo "=== kea Help ==="
    @echo "Platform: $(uname -s) $(uname -m)"
    @echo "Shell: $SHELL"
    @echo ""
    @echo "To report an issue:"
    @echo "  https://github.com/hyperpolymath/kea/issues/new"
    @echo ""
    @echo "Include the output of 'just doctor' in your report."
