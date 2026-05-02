# unbill-store-memory — Implementation

`InMemoryStore` holds a `Mutex<Inner>` where `Inner` contains a `HashMap<String, StoredLedger>` and a `HashMap<String, Vec<u8>>` for device metadata.

`StoredLedger` pairs a `LedgerMeta` with the serialized document bytes. Ledger bytes are stored as `Vec<u8>` (the output of `LedgerDoc::save`) and deserialized on each `load_ledger` call, keeping the round-trip semantics identical to `FsStore`.

There are no tests in this crate; it is exercised by `unbill-core` and `unbill-network` tests that use it as a dependency.
