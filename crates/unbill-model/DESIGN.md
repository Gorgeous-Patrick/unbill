# unbill-model

Domain types shared across all unbill crates. No business logic, no I/O.

## Types

- `Ulid` — typed identifier for ledgers, bills, and users; stored as a 26-character uppercase string
- `Timestamp` — Unix milliseconds; prevents passing arbitrary integers where a time value is expected
- `Currency` — ISO 4217 alphabetic code (e.g. `USD`); stored as a 3-letter string in documents
- `NodeId` — opaque string identity of a device; the network crate owns the conversion to/from Iroh types
- `SecretKey` — raw 32-byte Ed25519 key material; opaque to this crate
- `InviteToken` — 32-byte OS-random token, hex-encoded; held in memory only, never persisted or synced
- `Bill`, `NewBill`, `Share` — bill records and their construction input
- `EffectiveBills` — the subset of bills not superseded by a later amendment
- `Ledger`, `LedgerMeta`, `User`, `NewUser`, `Device`, `NewDevice`, `Invitation` — ledger and membership records
- `UnbillError`, `StorageError` — domain and storage error enumerations

## Rules

- no business logic; types are data carriers only
- types that appear in Automerge documents implement `autosurgeon::Reconcile` and `Hydrate`
- `NodeId` and `SecretKey` are opaque strings/bytes; conversion to network types lives outside this crate
