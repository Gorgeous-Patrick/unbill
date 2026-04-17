# unbill-tauri — Design Document

> Status: Stub (fill before M5)

## 1. Purpose

Thin Tauri 2 backend that exposes `UnbillService` as Tauri commands and forwards `ServiceEvent`s to the React frontend. Contains **no business logic**.

## 2. Public API sketch

One `#[tauri::command]` per `UnbillService` public method. Naming convention: `snake_case` on the Rust side, `camelCase` in JS via Tauri's automatic transformation.

Events emitted to the frontend:

```
unbill:ledger-updated   { ledger_id: string }
unbill:peer-connected   { ledger_id: string, peer: string }
unbill:peer-disconnected
unbill:sync-error       { ledger_id: string, peer: string, error: string }
```

## 3. Invariants

- Commands never block the main thread. All `UnbillService` calls are `async`.
- The `UnbillService` is initialized once in `setup` and shared via `tauri::State`.

## 4. Failure modes

- Tauri commands return `Result<T, String>` where the `String` is a human-readable error. The JS side handles these via try/catch on `invoke`.

## 5. Dependencies

| Crate | Why |
|-------|-----|
| `unbill-core` | all logic |
| `tauri` | desktop app shell |
| `tauri-plugin-shell` | shell integration for debug builds |

## 6. Testing strategy

- Manual testing of the full stack in M5.
- Unit tests of individual command handlers are low value; focus testing in `unbill-core`.

## 7. Open questions

- Mobile (iOS, Android): Tauri 2 supports it; defer to post-M5.
- Auto-updater: Tauri has a built-in updater. Opt-in only, per telemetry policy (DESIGN.md §10.3).
