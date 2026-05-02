# unbill-store-http

HTTPS-backed `LedgerStore` for the browser web frontend. Delegates all persistence to the unbill server over a REST API authenticated with a bearer token.

## REST API contract

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/api/v1/ledgers` | List ledger metadata |
| `PUT` | `/api/v1/ledgers/{id}/meta` | Save ledger metadata |
| `POST` | `/api/v1/ledgers/{id}/sync` | Automerge sync message exchange |
| `DELETE` | `/api/v1/ledgers/{id}` | Delete a ledger |
| `GET` | `/api/v1/device/id` | Get device node ID |
| `POST` | `/api/v1/device/key` | Initialize device secret key on server |
| `GET` | `/api/v1/device/{key}` | Load device metadata blob |
| `PUT` | `/api/v1/device/{key}` | Save device metadata blob |

## Sync protocol

`load_ledger` and `save_ledger` both run the Automerge sync loop: the client sends sync messages to `POST /sync` and receives server replies until both sides converge. The server holds the authoritative copy; the client merges in any server-side changes before returning, so `doc` reflects the converged state.

## Rules

- `get_secret_key` always returns `Err(StorageError::Unauthorized)`; the server holds the key material and never exposes it
- `load_ledger` returns `None` when the server responds 404 to the first sync message
- `delete_ledger` is idempotent; 404 is treated as success
- all requests carry a `Bearer` token; 401 surfaces as `StorageError::Unauthorized`
- compiles to `wasm32-unknown-unknown` for use in the browser frontend
