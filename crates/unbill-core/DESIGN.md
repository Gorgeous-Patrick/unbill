# unbill-core — Design Document

> Status: M0 complete. M1 (doc layer + service stubs) in progress.
> See project-level DESIGN.md for the full architecture.

## 1. Purpose

`unbill-core` is the library that defines what unbill *is*. It owns the CRDT document model, persistence, P2P networking, sync protocol, and all business logic (bill splitting, settlement). Every frontend (`unbill-cli`, `unbill-tauri`) is a thin consumer of this crate.

This crate does **not** contain CLI argument parsing, Tauri command wiring, or UI state.

## 2. Public API sketch

The primary entry point is `UnbillService`. See `service/service.rs` for the full method list. Key methods:

```rust
UnbillService::new(store: Arc<dyn LedgerStore>) -> Arc<Self>

// Ledger lifecycle
create_ledger(name: String, currency: Currency) -> Result<Ulid>
list_ledgers() -> Result<Vec<LedgerMeta>>
delete_ledger(ledger_id: &Ulid) -> Result<()>
export_ledger(ledger_id: &Ulid) -> Result<Vec<u8>>    // M3
import_ledger(bytes: &[u8]) -> Result<Ulid>            // M3

// Bills
add_bill(ledger_id: &Ulid, NewBill) -> Result<Ulid>
amend_bill(ledger_id: &Ulid, bill_id: &Ulid, BillAmendment) -> Result<()>
delete_bill(ledger_id: &Ulid, bill_id: &Ulid) -> Result<()>
list_bills(ledger_id: &Ulid) -> Result<Vec<EffectiveBill>>

// Members / settlement / events
list_members(ledger_id: &Ulid) -> Result<Vec<Member>>
compute_settlement(ledger_id: &Ulid) -> Result<Settlement>
subscribe() -> broadcast::Receiver<ServiceEvent>
```

Key model types exported from `unbill_core::model`:

| Type | Description |
|------|-------------|
| `Ulid` | Entity identifier (ledger, bill, user, amendment) |
| `Timestamp` | Unix-millisecond datetime newtype |
| `Currency` | ISO 4217 currency code (`iso_currency::Currency` wrapper) |
| `NodeId` | Iroh Ed25519 public key (`iroh::NodeId` wrapper) |
| `InviteToken` | 32-byte OS-random invite secret (in-memory only) |

## 3. Invariants

See project-level DESIGN.md §4.3 for the full invariant list. Summary:

- All entity IDs (`ledger_id`, `bill.id`, `user_id`, `amendment.id`) are `Ulid` — globally unique, monotonically ordered, never reused.
- Bills are append-only; logical deletion is tombstoning (`deleted: true`).
- Amendments are append-only; editing a bill means adding an `Amendment` record.
- `amount_cents` is non-negative.
- `Ledger.currency` is a valid `Currency` (ISO 4217) and is fixed at creation.
- `Device.node_id` and `Bill.created_by_device` are valid `NodeId`s (Ed25519 public keys).
- Member IDs are stable; adding a device appends to `member.devices`.
- `InviteToken` is 32 bytes from `OsRng`, hex-encoded — never stored to disk.

## 4. Failure modes

- `UnbillError::LedgerNotFound` — querying an unknown ledger ID.
- `UnbillError::InvalidInvitation` — expired or already-used join token.
- `UnbillError::NotAuthorized` — peer attempted to sync a ledger they're not a member of.
- `UnbillError::Storage(StorageError::Io(...))` — file I/O errors from `FsStore`.
- `UnbillError::Storage(StorageError::Serialization(...))` — corrupt or unreadable data files.

Callers (CLI, Tauri) are expected to map these to user-facing messages.

## 5. Dependencies

| Crate | Why |
|-------|-----|
| `automerge` | CRDT core |
| `autosurgeon` | ergonomic struct ↔ Automerge mapping |
| `iroh` | P2P transport; `iroh::NodeId` used in the model |
| `tokio` | async runtime; `tokio::fs` for flat-file storage |
| `ulid` | `Ulid` ID generation |
| `iso_currency` | `Currency` ISO 4217 enum |
| `rand` | `OsRng` for cryptographically secure `InviteToken` generation |
| `dashmap` | concurrent ledger map in `UnbillService` |
| `dirs` | platform data directory (`~/.local/share/unbill/` etc.) |
| `serde` / `serde_json` | `meta.json` and `snapshot_heads.json` serialization |
| `ciborium` | CBOR for binary message framing |
| `anyhow` / `thiserror` | error handling |
| `tracing` | structured logging |

## 6. Testing strategy

- Unit tests in each module, using `InMemoryStore` for storage-dependent tests.
- CRDT convergence tests with `proptest` — arbitrary operation interleavings, cross-merge, assert equal state.
- Sync protocol tests over in-process channels (no real Iroh endpoints).
- Settlement property tests: total owed = total paid, ≤ n-1 transactions.
- Storage round-trip tests: save → reload → assert identical doc bytes; compact → reload → assert identical.

## 7. Open questions

- **Lazy vs eager ledger loading at startup**: see project-level §14.
- **Sub-module DESIGN.md files**: each of `model/`, `doc/`, `storage/`, `net/`, `service/`, `settlement/` should get its own brief `DESIGN.md` before substantive implementation begins.
