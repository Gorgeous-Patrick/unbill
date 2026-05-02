# unbill-model — Implementation

Each type lives in its own module and is re-exported from `lib.rs`.

Types that appear in Automerge documents (`Ulid`, `Timestamp`, `Currency`, `NodeId`, `Bill`, `Share`, `Ledger`, `User`, `Device`) implement `autosurgeon::Reconcile` and `Hydrate` either via derive or manual impl.

`InviteToken` is memory-only and implements `serde::Serialize`/`Deserialize` for in-process use only — it is never written to a store or synced across devices.

`EffectiveBills` is a thin wrapper over `Vec<Bill>` returned by the service layer after filtering superseded bills.

`UnbillError` covers domain-level failures (ledger not found, user not in ledger, etc.). `StorageError` covers I/O and HTTP failures from store implementations and is separate so storage errors do not bleed into domain code.
