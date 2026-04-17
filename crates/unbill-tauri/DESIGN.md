# unbill-tauri

Thin Tauri 2 backend that exposes `UnbillService` as Tauri commands and forwards service events to the React frontend. Contains no business logic.

## Command surface

One Tauri command per `UnbillService` public method. Rust methods are `snake_case`; Tauri's automatic transformation makes them `camelCase` in JavaScript.

IDs (`ledger_id`, `bill_id`, `user_id`, `peer`) cross the Tauri boundary as ULID strings. The Rust side parses them back to typed values before calling into `UnbillService`.

## Events

The backend emits named events to the frontend whenever service state changes:

- `unbill:ledger-updated { ledger_id }` — any change to a ledger's content.
- `unbill:peer-connected { ledger_id, peer }` — a new sync peer appeared.
- `unbill:peer-disconnected { ledger_id, peer }` — a sync peer dropped.
- `unbill:sync-error { ledger_id, peer, error }` — a sync failure occurred.

## Invariants

- Commands never block the main thread. All `UnbillService` calls are async.
- `UnbillService` is initialized once in Tauri's `setup` hook and shared via `tauri::State`.
- Commands return a result where errors are human-readable strings. The JavaScript side handles them via try/catch on `invoke`.

## Open questions

- Mobile (iOS, Android): Tauri 2 supports it; deferred to post-M5.
- Auto-updater: Tauri has a built-in updater; opt-in only, per the telemetry policy in the root DESIGN.md.
