# SPDX-License-Identifier: AGPL-3.0-or-later
# Kea-Mandible build and run commands

# Default recipe - show available commands
default:
    @just --list

# Build all crates in debug mode
build:
    cargo build --workspace

# Build all crates in release mode
build-release:
    cargo build --workspace --release

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run clippy linter
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run all checks (lint, test, format)
check: fmt-check lint test

# Clean build artifacts
clean:
    cargo clean

# Deep audit of a target path
pry target:
    cargo run --release -- pry --target {{target}}

# WordPress audit
wordpress path:
    cargo run --release -- wordpress --path {{path}} --audit-config

# Bloat analysis
slop target:
    cargo run --release -- slop --target {{target}}

# Run with verbose output
pry-verbose target:
    cargo run --release -- --verbose pry --target {{target}} --include-hidden

# Generate documentation
doc:
    cargo doc --workspace --no-deps --open

# Install locally
install:
    cargo install --path crates/kea-mandible

# Show version
version:
    cargo run -- version
