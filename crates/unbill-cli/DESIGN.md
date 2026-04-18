# unbill-cli

A thin clap-driven command-line frontend for `UnbillService`. Useful for dogfooding, automated testing, and terminal users. Contains no business logic — all work is delegated to `unbill-core`.

## Commands

- `init new <display_name>` — generate a device key and a fresh user identity (user ID + display name). Stored as device-local metadata.
- `init import <url>` — generate a device key and import an existing user identity from another device via an `unbill://identity/...` URL. The other device must be online.
- `device show | share | remove` — `device share` generates an `unbill://identity/...` URL so another device can import this device's user identity via `init import`. `device remove --ledger <ledger_id> --node-id <node_id>` removes an authorized device from a ledger (any trusted device may remove any other).
- `ledger create | list | show | delete | invite | join` — ledger lifecycle. `ledger create` registers the creator's own device in `ledger.devices`. `ledger invite` generates an `unbill://join/...` URL authorizing a new device to access the ledger; `ledger join <url>` accepts one.
- `bill add | list | amend | delete | restore` — bill management. `--added-by` defaults to the local user ID at the CLI level.
- `member list | add | remove` — managing named participants in a ledger.
- `sync daemon | once | status` — P2P sync control. `sync once <peer_node_id>` dials a specific peer and syncs; `sync daemon` opens the endpoint and waits for incoming connections.
- `settlement <user_id>` — display who owes whom for a user, aggregated across all their ledgers.

Ledger and bill IDs are ULID strings on the command line. Most commands accept `--json` for machine-readable output, used in end-to-end tests.

## Invariants

- The binary never touches storage or network directly. All side effects go through `UnbillService`.
- Exit code 0 on success, non-zero on any error. Error messages go to stderr.

## Failure modes

- `UnbillError` variants are mapped to human-readable stderr messages.
- `sync once` exits non-zero if the peer is unreachable.
- `member join` exits non-zero if the host is offline or the token is invalid.
