# storage — Implementation

`traits.rs` defines the `LedgerStore` interface. `fs.rs` implements it with flat files and atomic overwrite semantics. `memory.rs` implements it for tests. `http.rs` implements it against a remote HTTPS server.

`FsStore` writes `ledger.bin` and `meta.json` under a per-ledger directory and uses top-level device metadata files for keys, saved users, labels, and pending tokens. Reads return empty values for missing optional data so callers can treat first-run state as normal.

`HttpStore` wraps a `reqwest::Client` and maps each `LedgerStore` method to one HTTP request as described in `DESIGN.md §REST API contract`. It reuses the same `MetaJson` serde struct as `FsStore`. 404 responses are silently converted to the missing-value sentinel (empty `Vec` or `None`) matching the rest of the trait's contract. Other non-2xx responses surface as `StorageError`.

## Dependencies

- `reqwest` (with `json` and `rustls-tls` features, no default features) for the HTTP client
- `wiremock` (dev-dependency) for HTTP mock server in tests
