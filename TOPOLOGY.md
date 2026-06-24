<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-03-16 -->

# Kea — Project Topology

## System Architecture

```
                        ┌─────────────────────────────────────────┐
                        │              OPERATOR / ADMIN           │
                        │        (Playbooks, CLI, Dashboard)      │
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │           KEA-BIVOUAC (CORE)            │
                        │    (Command Authority, Orchestrator)    │
                        │    mTLS zero-trust, playbook executor   │
                        └──────────┬───────────────────┬──────────┘
                                   │                   │
                                   ▼                   ▼
                        ┌───────────────────────┐  ┌────────────────────────────────┐
                        │ KEA-CALL (SIGNALLING) │  │ KEA-WIT (INTERFACE)            │
                        │ - Cap'n Proto Defs    │  │ - WIT Definitions              │
                        │ - MCP Communication   │  │ - Component Model Tooling      │
                        │ - Request-Signal-Act  │  │ - WASM Validation              │
                        └──────────┬────────────┘  └──────────┬─────────────────────┘
                                   │                          │
                                   └────────────┬─────────────┘
                                                ▼
                        ┌─────────────────────────────────────────┐
                        │           KEA-MANDIBLE (SENSORS)        │
                        │  ┌───────────┐  ┌───────────┐  ┌───────┐│
                        │  │ Kea-Beak  │  │ WP-Praxis │  │ Slop  ││
                        │  │ (FS/Net)  │  │(WordPress) │  │ Gate  ││
                        │  │ 10k f/sec │  │ Core Audit │  │ Bloat ││
                        │  └───────────┘  └───────────┘  └───────┘│
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │          TARGET INFRASTRUCTURE          │
                        │      (Servers, Networks, App Cores)     │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          ABI / FFI / API LAYER          │
                        │  Idris2 (ABI)  Zig (FFI)  zig (API) │
                        │         (Planned — Phase 2)             │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          REPO INFRASTRUCTURE            │
                        │  17 CI/CD Workflows    .machine_readable│
                        │  Justfile Automation   Chainguard Images│
                        │  panic-attack Scans    Hypatia Security │
                        └─────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
CORE ECOSYSTEM
  Kea-Bivouac (Orchestrator)        ██████████ 100%    Command authority stable, mTLS ready
  Kea-Call (Signalling)             ██████████ 100%    Zero-copy Cap'n Proto active
  Kea-Mandible (Sensors)            ████████░░  80%    Investigation logic refining
  Kea-Wit (WASM Interfaces)         ███████░░░  70%    Spec in progress, ROADMAP.adoc

MANDIBLE CRATES
  Kea-Beak (FS/Network Probe)      ██████████ 100%    10k files/sec, SHA256+BLAKE3
  Kea-Mandible CLI                  ████████░░  80%    Functional, needs output polish
  WP-Praxis (WordPress Audit)      ████████░░  80%    Core audit works, edge cases remain
  Slop-Gate (Bloat Detection)       ██████░░░░  60%    Initial heuristics, needs tuning

REPO INFRASTRUCTURE
  CI/CD Workflows (17/17)           ██████████ 100%    Full RSR standard
  .machine_readable/ State          ██████████ 100%    A2ML format, 6 files
  Justfile Automation               ██████████ 100%    Build/test/lint/audit recipes
  .well-known/ Discovery            ██████████ 100%    security.txt, ai.txt, humans.txt
  .github/ Community Files          ██████████ 100%    CODEOWNERS, templates, dependabot

ABI / FFI / API (PLANNED)
  Idris2 ABI Definitions            ░░░░░░░░░░   0%    Not started — Phase 2
  Zig FFI Implementation            ░░░░░░░░░░   0%    Not started — Phase 2
  zig API Connectors             ░░░░░░░░░░   0%    Not started — Phase 2

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            █████████░  ~85%   MVP target: Slop-Gate + WIT + E2E
```

## Data Flow

```
Kea-Mandible                  Kea-Call                    Kea-Bivouac
(Sensor Suite)                (Protocol)                  (Orchestrator)
    │                             │                           │
    │  1. Probe target FS/net     │                           │
    │  2. Collect findings        │                           │
    │───── Cap'n Proto ──────────►│                           │
    │                             │  3. Route signal          │
    │                             │───── MCP/Action ─────────►│
    │                             │                           │  4. Execute playbook
    │                             │                           │  5. Deploy fix/alert
    │◄────────────── Feedback ────┤◄──────────────────────────│
    │  6. Re-probe to verify      │                           │
```

## Subproject Index

| Component | Directory | Language | Status | MVP Gap |
|-----------|-----------|----------|--------|---------|
| Kea-Bivouac | `bivouac/` | Rust | Stable | None |
| Kea-Call | `call/` | Cap'n Proto/MCP | Stable | None |
| Kea-Beak | `mandible/crates/kea-beak/` | Rust | Stable | None |
| Kea-Mandible CLI | `mandible/crates/kea-mandible/` | Rust | 80% | Output formats, help text |
| WP-Praxis | `mandible/crates/wp-praxis/` | Rust | 80% | Multisite, custom themes |
| Slop-Gate | `mandible/crates/slop-gate/` | Rust | 60% | Heuristic tuning, thresholds |
| Kea-Wit | `wit/` | WIT | 70% | Formal spec (see ROADMAP.adoc) |

## Shortest Route to MVP

1. **Slop-Gate** (60% → 90%): Tune heuristics, add configurable thresholds, test against real hosting dirs
2. **WP-Praxis** (80% → 95%): WordPress multisite support, custom theme detection
3. **Kea-Wit** (70% → 90%): Formalise WIT interfaces for Mandible ↔ Bivouac pipeline
4. **Kea-Mandible CLI** (80% → 95%): JSON/TOML output formats, improved help text
5. **E2E Integration**: Full Mandible → Call → Bivouac pipeline test
6. **Container Build**: Chainguard-based OCI image for deployment

## Phase 2 (Post-MVP)

- Add Idris2 ABI definitions in `src/abi/` for cross-component type safety
- Add Zig FFI layer in `ffi/zig/` for C-compatible sensor plugins
- Publish zig API connectors from `developer-ecosystem/v-ecosystem/`
- BoJ-server integration: Kea sensors as MCP cartridge data sources

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
