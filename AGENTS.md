# Unbill — Engineering Skills and Workflows

## Design-first development

Every non-trivial piece of functionality begins with a design document, not code. The order is:

1. Write or update DESIGN.md (and IMPLEMENTATION.md if needed) before any production code.
1. Write failing tests before implementing.
1. Implement until tests pass.
1. Refactor with tests as a safety net.

No production code ships without a prior failing test. Exceptions are type definitions, `todo!` stubs, and module declarations.

## Test-first development

Tests are written before or alongside implementation, never after. Co-locate tests in `#[cfg(test)]` modules at the bottom of the file they cover.

**Test names describe behavior, not implementation.** `test_settlement_balances_to_zero` not `test_compute_settlement`.

Priority order for writing tests:

1. Pure functions with no I/O (settlement, projection logic) — easiest to specify and verify.
1. Storage layer — save/load/compact round-trips with `InMemoryStore`.
1. CRDT operations — convergence after arbitrary operation interleavings, using `proptest`.
1. Sync protocol — in-process channels simulating the network; no real Iroh endpoints.
1. CLI end-to-end — shell scripts in `tests/e2e/` against real temp directories.

## Documentation rules

- DESIGN.md always reflects the current intended design. No history, no "we used to do X." When a decision changes, update the document and remove the old description.
- IMPLEMENTATION.md always reflects how the current design is implemented. No rationale for discarded approaches.
- Docs and code change in the same commit. Drift between design and implementation is worse than no docs at all.
- When an open question is resolved, fold the answer into the relevant section and delete the question.
- No code in documentation files. Code changes frequently; docs should stay stable and conceptual.

## Crate and module documentation

Each crate has a DESIGN.md (what it is, why it exists, its contract) and an IMPLEMENTATION.md (how it is built, what it depends on, how it is tested). These must exist before substantial implementation begins.

Sub-modules that own a significant design surface (e.g., `storage/`, `net/`, `settlement/`) get their own DESIGN.md and IMPLEMENTATION.md under `src/<module>/`.
