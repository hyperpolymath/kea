# TEST-NEEDS.md — kea

> Generated 2026-03-29 by punishing audit.

## CRG Grade: C — ACHIEVED 2026-04-04

All CRG C requirements met for the `bivouac` crate:
- Unit tests: 22 inline `#[test]` in `src/config.rs` and `src/playbook/` (100% pass)
- Smoke tests: integration via CLI binary tests
- P2P/property-based: 5 proptest tests in `bivouac/tests/property_tests.rs` (100% pass)
- E2E/reflexive: 5 tests in `bivouac/tests/e2e_test.rs` (100% pass)
- Contract tests: 11 tests in `bivouac/tests/contract_tests.rs` (100% pass)
- Aspect tests: 10 tests in `bivouac/tests/aspect_tests.rs` (100% pass)
- Benchmarks: Criterion benchmarks in `bivouac/benches/bivouac_bench.rs`

## Current State

| Category     | Count | Notes |
|-------------|-------|-------|
| Unit tests   | 0     | No inline `#[test]` in source modules |
| Integration  | 1     | bivouac/tests/integration_test.rs |
| E2E          | 0     | None |
| Benchmarks   | 0     | None |

**Source modules:** ~11 Rust source files across 2 subsystems: bivouac (playbook executor/parser, config, error, lib, main) and mandible (kea-beak, kea-mandible, slop-gate, wp-praxis). Fuzz targets exist in both.

## What's Missing

### P2P (Property-Based) Tests
- [ ] Playbook parser: arbitrary YAML/config input fuzzing
- [ ] Executor: property tests for idempotent operations
- [ ] Config validation: arbitrary config structure testing
- [ ] slop-gate: input validation property tests

### E2E Tests
- [ ] Full playbook execution: parse -> validate -> execute -> verify
- [ ] mandible pipeline: kea-beak -> kea-mandible -> wp-praxis flow
- [ ] Error recovery: interrupted playbook execution and resume

### Aspect Tests
- **Security:** No tests for playbook injection, path traversal in executor, untrusted input handling
- **Performance:** No execution time benchmarks
- **Concurrency:** No tests for parallel playbook execution
- **Error handling:** No tests for malformed playbooks, missing dependencies, partial execution failure

### Build & Execution
- [ ] `cargo test` across all crates
- [ ] `cargo fuzz` for both fuzz targets

### Benchmarks Needed
- [ ] Playbook parsing speed vs complexity
- [ ] Executor throughput (operations/second)
- [ ] wp-praxis processing latency

### Self-Tests
- [ ] Playbook schema self-validation
- [ ] Dependency resolution verification

## Priority

**CRITICAL.** 11 source modules with 0 unit tests and 1 integration test. The mandible subsystem (4 crates) has ZERO tests of any kind. Fuzz targets exist but that is not a substitute for structured tests.

## FAKE-FUZZ ALERT

- `tests/fuzz/placeholder.txt` is a scorecard placeholder inherited from rsr-template-repo — it does NOT provide real fuzz testing
- Replace with an actual fuzz harness (see rsr-template-repo/tests/fuzz/README.adoc) or remove the file
- Priority: P2 — creates false impression of fuzz coverage
