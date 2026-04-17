# unbill-cli — Design Document

> Status: Stub (fill before M2)

## 1. Purpose

A thin `clap`-driven command-line frontend that drives `UnbillService`. Useful for dogfooding, automated testing, and users who prefer the terminal. Contains **no business logic** — all work is delegated to `unbill-core`.

## 2. Public API sketch

No library API. Binary only: `unbill <subcommand>`.

```
unbill init
unbill ledger create|list|show|export|import|delete
unbill bill add|list|amend|delete|restore
unbill member list|invite|join
unbill sync daemon|once|status
unbill settlement show
```

Most commands accept `--json` for machine-readable output (used in e2e tests).

## 3. Invariants

- The binary never touches storage or network directly. All side effects go through `UnbillService`.
- Exit code 0 on success, non-zero on error. Error messages printed to stderr.

## 4. Failure modes

- `UnbillError` variants are mapped to human-readable stderr messages.
- Network timeouts in `sync once` print a warning and exit non-zero.

## 5. Dependencies

| Crate | Why |
|-------|-----|
| `unbill-core` | all business logic |
| `clap` | argument parsing |
| `tokio` | async runtime for service calls |
| `tracing-subscriber` | log output |
| `anyhow` | error propagation |

## 6. Testing strategy

- Shell-script end-to-end tests under `tests/e2e/` (see project DESIGN.md §9.4).
- No unit tests in the CLI itself — logic lives in `unbill-core`.

## 7. Open questions

- Output format for `bill list`: table (human) vs `--json`; which fields to show by default?
- `sync daemon`: should it daemonize (fork + setsid) or run in the foreground? Foreground first; daemonize later if there's demand.
