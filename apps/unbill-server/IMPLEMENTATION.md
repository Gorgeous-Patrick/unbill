# unbill-server — Implementation

The binary is built from `src/main.rs`, which reads environment variables, creates an `FsStore`, and starts an `axum` HTTP server.

The router lives in `src/router.rs` and is exported by `src/lib.rs` so that integration tests can call it without a real TCP socket.

## Module layout

- `src/main.rs` — reads config, builds the router, binds TCP, runs the server
- `src/lib.rs` — exports `build_router` and `AppState`
- `src/router.rs` — route table, handlers, auth middleware

## Auth middleware

A `tower` middleware function (`auth`) runs before every handler. It extracts the `Authorization` header, checks the `Bearer` scheme, and compares the token to `AppState::api_key`. Mismatches return `401` before reaching any handler.

## Handlers

Each handler receives `State<Arc<AppState>>` (containing the `FsStore` and `api_key`) and calls the corresponding `LedgerStore` method. Storage errors map to HTTP responses:

- `StorageError::Io` with `NotFound` is already converted to `None`/empty by the store, so handlers never see it.
- Other `StorageError` variants become `500 Internal Server Error` with the error message in the body.

## Device key validation

A small helper `valid_device_key(key)` returns `false` if the key contains anything other than `[a-zA-Z0-9._-]`. Handlers return `400` for invalid keys before touching the store.

## Dependencies

- `axum 0.8` — routing and extractors
- `tower 0.5` — middleware
- `tower-http` — request tracing via `TraceLayer`
- `clap` is not used; configuration is env-only for simplicity
