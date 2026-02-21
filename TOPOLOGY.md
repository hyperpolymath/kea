<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-02-19 -->

# Kea-Tools — Project Topology

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
                        └──────────┬───────────────────┬──────────┘
                                   │                   │
                                   ▼                   ▼
                        ┌───────────────────────┐  ┌────────────────────────────────┐
                        │ KEA-CALL (SIGNALLING) │  │ KEA-WIT (INTERFACE)            │
                        │ - Cap'n Proto Defs    │  │ - WIT Definitions              │
                        │ - MCP Communication   │  │ - Component Model Tooling      │
                        └──────────┬────────────┘  └──────────┬─────────────────────┘
                                   │                          │
                                   └────────────┬─────────────┘
                                                ▼
                        ┌─────────────────────────────────────────┐
                        │           KEA-MANDIBLE (SENSORS)        │
                        │  ┌───────────┐  ┌───────────┐  ┌───────┐│
                        │  │ Kea-Beak  │  │ WP-Praxis │  │ Slop  ││
                        │  │ (FS/Net)  │  │ (WordPress)│  │ Gate  ││
                        │  └───────────┘  └───────────┘  └───────┘│
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │          TARGET INFRASTRUCTURE          │
                        │      (Servers, Networks, App Cores)     │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          REPO INFRASTRUCTURE            │
                        │  Justfile / Cargo   .machine_readable/  │
                        │  Cap'n Proto        Deno Runtime        │
                        └─────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
CORE ECOSYSTEM
  Kea-Bivouac (Orchestrator)        ██████████ 100%    Command authority stable
  Kea-Call (Signalling)             ██████████ 100%    Zero-copy Cap'n Proto active
  Kea-Mandible (Sensors)            ████████░░  80%    Investigation logic refining
  Kea-Wit (WASM Interfaces)         ██████████ 100%    Component model stable

TOOLS & PLUGINS
  Kea-Beak (Network Probe)          ██████████ 100%    Metadata probing verified
  WP-Praxis Integration             ████████░░  80%    WordPress audit refining
  Slop-Gate (Bloat Detection)       ██████░░░░  60%    Initial heuristics active

REPO INFRASTRUCTURE
  Justfile Automation               ██████████ 100%    Standard build/test tasks
  .machine_readable/                ██████████ 100%    STATE tracking active
  Monorepo Management               ██████████ 100%    Inter-component dependencies stable

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            █████████░  ~90%   v1.0.0 Production Ready
```

## Key Dependencies

```
Kea-Wit (Logic) ──────► Kea-Call ──────► Kea-Bivouac ──────► Deployment
     │                    ▲                 │                   │
     ▼                    │                 ▼                   ▼
Mandible (Sensor) ────────┘           Target Infra ◄──────── Feedback
```

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
