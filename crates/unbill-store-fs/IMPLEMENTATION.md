# unbill-store-fs — Implementation

`FsStore` is a newtype around a `PathBuf` root directory. Every `LedgerStore` method maps directly to filesystem operations via `tokio::fs`.

`MetaJson` is a plain-primitives mirror of `LedgerMeta` used for JSON serialization, avoiding domain-type imports in the serialization layer.

`atomic_write` writes to a `.tmp` sibling and renames into place to prevent partial writes from leaving a corrupt file behind.

`UnbillPath` and the `UNBILL_PATH` constant (in `path.rs`) resolve the platform default data directory (`~/.local/share/unbill` on Linux, `~/Library/Application Support/unbill` on macOS) and expose an `ensure_data_dir` helper used by desktop and server entry points.

Tests use `tempfile::tempdir()` to isolate each test to a fresh directory. Coverage includes save/list/load/delete round-trips for ledger data and device metadata.
