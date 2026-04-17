# unbill-tauri — Implementation Notes

## Dependencies

| Crate | Why |
|-------|-----|
| `unbill-core` | All business logic |
| `tauri` | Desktop app shell and IPC bridge |
| `tauri-plugin-shell` | Shell integration for debug builds |

## Testing strategy

Individual command handler unit tests are low value — logic lives in `unbill-core`. Full-stack testing is manual at M5, covering the Tauri → React round-trip for each command and each event type.
