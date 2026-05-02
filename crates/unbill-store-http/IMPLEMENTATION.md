# unbill-store-http — Implementation

`HttpStore` holds a `reqwest::Client`, a `base_url` (normalized to `{origin}/api/v1`), and an `api_key`. Every request is authorized via `Bearer` token through the `auth` helper.

`run_sync_loop` drives the Automerge sync protocol: it calls `doc.generate_sync_message`, POSTs the encoded bytes to `/ledgers/{id}/sync`, and feeds the server's response back into `doc.receive_sync_message`, repeating until no more messages are produced. `204 No Content` from the server means it has nothing to send for this round; `404` on the first message means the ledger does not exist on the server. Both `load_ledger` and `save_ledger` run this loop, so saves also pull in any concurrent server-side changes.

`MetaJson` is a plain-primitives mirror of `LedgerMeta` used for JSON serialization, identical in structure to the one in `unbill-store-fs`.

The `check` helper maps 401 to `StorageError::Unauthorized` and other non-2xx responses to `StorageError::HttpStatus`.

Tests use `wiremock` to mock the HTTP server and verify request shape (method, path, bearer token, content-type) and response handling, including sync convergence, 401 and 500 error surfaces, and 404 on load.
