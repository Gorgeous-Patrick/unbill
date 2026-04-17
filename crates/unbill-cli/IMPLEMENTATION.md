# unbill-cli — Implementation Notes

## Dependencies

| Crate | Why |
|-------|-----|
| `unbill-core` | All business logic |
| `clap` | Argument parsing and subcommand dispatch |
| `tokio` | Async runtime for service calls |
| `tracing-subscriber` | Log output formatting |
| `anyhow` | Error propagation |

## Testing strategy

Shell-script end-to-end tests under `tests/e2e/`. Each test creates temporary data directories for two simulated devices, runs realistic CLI scenarios, and asserts final state via `--json` output. No unit tests in the CLI itself — logic lives in `unbill-core`.
