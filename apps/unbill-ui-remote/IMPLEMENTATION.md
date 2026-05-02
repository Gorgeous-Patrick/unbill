# unbill-ui-remote Implementation

The app is a client-side rendered Leptos application compiled to WASM with Trunk. `main.rs` initializes the `UnbillService` and mounts `App`. `app.rs` owns navigation state and calls into `api.rs`. `pages.rs` defines screen-level components. `api.rs` wraps `UnbillService` calls and maps results to plain DTO structs.

## Service initialization

`main.rs` reads `UNBILL_SERVER_URL` (baked in at compile time via `env!`), constructs an `HttpStore` with that base URL, and calls `UnbillService::open(store)`. The resulting service is passed into `App` as a prop and stored in a context so all descendant components can reach it.

## State model

The app keeps backend data in signals: bootstrap state for the device ID, ledger list, and all users; selected ledger detail for bills, users, and settlement; settings overlay ledger detail for overlay-only ledger selection; and transient overlay state for create, invite, add-user, and bill editor flows. Mutating actions call `api.rs`, show shared status or error feedback, refresh bootstrap state, and refresh the selected ledger detail when the visible ledger could have changed.

## API layer

`api.rs` defines async functions that delegate to `UnbillService` and return typed DTO structs. DTOs are plain `serde` structs defined alongside the functions. `main.rs` and `app.rs` call into `api.rs`; no component accesses the service directly.

## Navigation and settings

Settings state is a single popup state value holding the active tab and selected ledger ID. In ranger mode the popup overlays three columns. In compact mode it fills the viewport. Responsive mode selection uses the same 1200 px breakpoint and window resize listener as the shared UI model.

Device Settings renders the server-assigned device ID (read-only). Ledger Settings renders a ledger selector, ledger users, an add-user picker, and invitation URL generation. There are no sync controls.

## Clipboard

Invitation URLs are written via the browser `navigator.clipboard.writeText` API through a `web-sys` binding.

## Stylesheet

System typography, full-height panes, dense rows, compact toolbars, restrained borders, stable control dimensions. Navigation and utility controls use Lucide SVG icon buttons. Primary workflow actions use text buttons.

## Dependencies

- `leptos` (csr feature)
- `unbill-core` (no `network` feature)
- `unbill-store-http`
- `wasm-bindgen`, `wasm-bindgen-futures`
- `web-sys` (clipboard, window, event)
- `unbill-ui-components`
- `serde`, `serde_json`
- `console_error_panic_hook`

## Build

`trunk build` produces a static bundle in `dist/`. `UNBILL_SERVER_URL` must be set in the environment before building. `trunk serve` rebuilds on change and proxies to the same origin by default.

## Tests

Pure state helpers (DTO mapping, navigation state transitions) are unit-tested in `#[cfg(test)]` modules in `api.rs` and `app.rs`.
