# SPDX-License-Identifier: PMPL-1.0-or-later
# Kea-Tools: Unified build and run commands for the Kea ecosystem

# Default recipe - show available commands
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

# === Call commands ===

# Generate language-specific bindings from Cap'n Proto schemas
call-generate-bindings:
    cd call && just generate-bindings

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

# === Wit commands ===

# Build Wit tooling
wit-build:
    cd wit && echo "Wit tooling - specification pending"

# === Documentation ===

# Generate documentation for all Rust components
doc:
    cd bivouac && cargo doc --workspace --no-deps
    cd mandible && cargo doc --workspace --no-deps
