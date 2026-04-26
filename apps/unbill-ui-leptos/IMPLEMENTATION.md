# Unbill UI Leptos Implementation

The Leptos app is a client-side rendered Tauri frontend. `main.rs` mounts `App`, `app.rs` owns navigation state and async bridge calls, `pages.rs` defines screen-level components, `components.rs` contains reusable UI pieces, and `api.rs` mirrors the JSON DTOs returned by Tauri commands.

The app keeps backend data in signals: bootstrap state for ledgers, local users, and known devices; selected ledger detail for bills, users, settlement, and ledger-scoped sync peers; and transient overlay state for create, join, invite, and editor flows. Mutating actions call the bridge, show shared status or error feedback, refresh bootstrap state, and refresh selected ledger detail when the active ledger could have changed.

Device Settings renders all known peer devices across local ledgers. Ledger Settings renders only the peer devices authorized for the selected ledger, and each peer row triggers the same one-shot sync bridge command used by Device Settings.

Tests live beside the Rust code they cover. Pure UI helpers are unit-tested in the Leptos crate when behavior can be isolated without a browser, while bridge DTO assembly is tested in the Tauri crate.
