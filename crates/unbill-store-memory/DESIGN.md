# unbill-store-memory

In-memory `LedgerStore` for unit tests. Stores ledger data and device metadata in `HashMap`s behind a `Mutex`. Not intended for production use.

## Rules

- all operations are synchronous under the lock; no I/O
- `save_ledger` serializes the document to bytes and back on load, exercising the same round-trip path as real stores
- `create_secret_key` is idempotent; it does not overwrite an existing key
- `get_secret_key` is supported (unlike `HttpStore`)
