# unbill-tui — Implementation

## Crate structure

```
src/
├── main.rs       — entry point: open service, run event loop
├── app.rs        — AppState and top-level event dispatch
├── ui.rs         — top-level render function; composes panes and status bar
├── pane/
│   ├── mod.rs    — Pane enum (Ledger, Bills, Detail)
│   ├── ledger.rs — render and keymap for the ledger list pane
│   └── bills.rs  — render and keymap for the bill list pane
└── modal/
    ├── mod.rs        — Modal trait and ModalStack
    ├── create_ledger.rs
    ├── add_bill.rs
    ├── amend_bill.rs
    ├── users.rs
    ├── settlement.rs
    ├── invite.rs
    └── confirm.rs
```

## Dependencies

- `ratatui` — terminal rendering
- `crossterm` — terminal backend and raw-mode input
- `tokio` — async runtime (shared with unbill-core)
- `unbill-core` — all domain logic via `UnbillService`

## AppState

`AppState` is the single source of UI state. It holds no ledger data directly — all domain data is fetched from the service on demand and cached only for the current render frame.

```
focused_pane: Pane
ledger_cursor: usize        // index into the fetched ledger list
bill_cursor: usize          // index into the fetched bill list
modal: Option<Box<dyn Modal>>
sync_status: SyncStatus     // Idle | Syncing | Error(String)
status_message: Option<String>  // transient error or info line
```

When an action mutates the ledger, `AppState` calls the service and then clears any cached data so the next render re-fetches.

## Event loop

The main loop runs inside a single tokio task and drives three concurrent streams:

1. **Terminal events** — crossterm key/resize events via `EventStream`.
2. **Service events** — `broadcast::Receiver<ServiceEvent>` from `UnbillService::subscribe()`.
3. **Render tick** — a fixed-interval ticker (16 ms, ~60 fps) that triggers a redraw.

On each iteration the loop selects across all three streams, updates `AppState`, then redraws. Service events that signal `LedgerUpdated` clear the bill cache so the next render fetches fresh data.

## Rendering

`ui.rs` calls `ratatui`'s `Layout::horizontal` to split the terminal into three equal-ish columns (roughly 20 % / 40 % / 40 %). Each pane module exposes a `render(frame, area, state, data)` function that draws into its allocated area. The status bar occupies a fixed one-line area at the bottom.

Pane borders are styled to distinguish focused (bright) from unfocused (dim). The cursor row within a list is highlighted with a reversed-video style.

## Modal system

`Modal` is a trait with two methods:

- `render(&self, frame, area)` — draws the modal centered over the screen.
- `handle_key(&mut self, key) -> ModalOutcome` — returns `Pending`, `Confirmed(ModalResult)`, or `Cancelled`.

`ModalResult` is an enum carrying the data needed for the service call (e.g. `CreateLedger { name, currency }`). After a `Confirmed` result, the event loop extracts the result, calls the appropriate service method, and drops the modal.

Modals with multiple fields track a `focused_field: usize` internally and advance it on `Tab`.

## Status bar hints

Each `Pane` variant exposes a `hints() -> &[(&str, &str)]` function returning `(key, label)` pairs. The status bar renders these as `[key] label` separated by spaces, rebuilding the string only when focus changes.

## Testing

The TUI has no unit tests of its own. Correctness of domain logic is covered by `unbill-core` tests. The TUI is validated manually and via the existing CLI e2e tests which exercise the same service layer.
