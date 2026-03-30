# PROOF-NEEDS.md — kea

## Current State

- **src/abi/*.idr**: NO
- **Dangerous patterns**: 54 `unwrap()` calls in Rust code
- **LOC**: ~3,300 (Rust)
- **ABI layer**: Missing

## What Needs Proving

| Component | What | Why |
|-----------|------|-----|
| WordPress praxis (wp-praxis) | Input sanitisation correctness | WP integration handles untrusted user input |
| Slop gate filtering | Content filter decisions are deterministic | Incorrect filtering passes or blocks wrong content |
| Bivouac deployment | Deployment state machine correctness | Bad state transitions leave infrastructure in broken state |
| Kea-beak API | API request/response contract adherence | Malformed responses break downstream consumers |

## Recommended Prover

**Idris2** — Create `src/abi/` with types for deployment state machine and content filtering invariants. Small enough codebase to prove completely.

## Priority

**LOW** — Infrastructure orchestration toolkit. The 54 unwrap() calls indicate error handling gaps, but the blast radius is limited to infrastructure tooling.
