# unbill-server-client

Typed async HTTP client for the operational endpoints of `unbill-server` that are not part of the `LedgerStore` trait: peer sync, ledger invite creation, and ledger join.

## Purpose

`unbill-store-http` covers ledger persistence. This crate covers the three network/identity endpoints that coordinate device-to-device sync and onboarding:

- Trigger Iroh P2P sync between the server and a known peer.
- Create a one-time join invitation for a ledger (host side).
- Join a ledger hosted on another device (joiner side).

Any Rust client that needs to drive these flows — CLI, TUI, web frontend — imports this crate instead of hand-rolling `reqwest` calls.

## API

```
ServerClient::new(base_url, api_key) -> ServerClient

async fn sync_with_peer(node_id: &str)           -> Result<(), ServerClientError>
async fn create_invitation(ledger_id: &str)       -> Result<String, ServerClientError>  // invite URL
async fn join_ledger(url: &str, label: Option<&str>) -> Result<(), ServerClientError>
```

All methods require an `Authorization: Bearer` token identical to the one used by `HttpStore`. A missing or wrong key surfaces as `ServerClientError::Unauthorized`.

## Endpoints called

| Method | Path | Used by |
|--------|------|---------|
| `POST` | `/api/v1/peers/{node_id}/sync` | `sync_with_peer` |
| `POST` | `/api/v1/ledgers/{id}/invitations` | `create_invitation` |
| `POST` | `/api/v1/ledgers/join` | `join_ledger` |

## Error model

`ServerClientError` covers: `Unauthorized` (401), `NotFound` (404), `ServiceUnavailable` (503 — device not yet initialised), `Network` (transport failure), `HttpStatus` (any other non-success status with body text).

## Compatibility

Compiles to `wasm32-unknown-unknown` for use in the browser frontend, using the `wasm`-compatible `reqwest` feature set.
