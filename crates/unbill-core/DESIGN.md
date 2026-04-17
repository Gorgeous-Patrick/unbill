# unbill-core — Design Document

> Status: Stub (fill before M1)
> See project-level DESIGN.md for the full architecture.

## 1. Purpose

`unbill-core` is the library that defines what unbill *is*. It owns the CRDT document model, persistence, P2P networking, sync protocol, and all business logic (bill splitting, settlement). Every frontend (`unbill-cli`, `unbill-tauri`) is a thin consumer of this crate.

This crate does **not** contain CLI argument parsing, Tauri command wiring, or UI state.

## 2. Public API sketch

The primary entry point is `UnbillService`. See `service/service.rs` for the full method list. Key methods:

```rust
UnbillService::new(store: Arc<dyn LedgerStore>) -> Arc<Self>

// Ledger lifecycle
create_ledger(name, currency) -> Result<String>
list_ledgers() -> Result<Vec<LedgerMeta>>
delete_ledger(ledger_id) -> Result<()>
export_ledger(ledger_id) -> Result<Vec<u8>>
import_ledger(bytes) -> Result<String>

// Bills
add_bill(ledger_id, NewBill) -> Result<String>
amend_bill(ledger_id, bill_id, BillAmendment) -> Result<()>
delete_bill(ledger_id, bill_id) -> Result<()>
list_bills(ledger_id) -> Result<Vec<EffectiveBill>>

// Members / settlement / events
list_members(ledger_id) -> Result<Vec<Member>>
compute_settlement(ledger_id) -> Result<Settlement>
subscribe() -> broadcast::Receiver<ServiceEvent>
```

## 3. Invariants

See project-level DESIGN.md §4.3 for the full invariant list. Summary:

- Bill IDs are ULIDs, globally unique, never reused.
- Bills are append-only; logical deletion is tombstoning (`deleted: true`).
- Member IDs are stable; adding a device appends to `member.devices`.
- `amount_cents` is non-negative.
- Ledger currency is fixed at creation.

## 4. Failure modes

- `UnbillError::LedgerNotFound` — querying an unknown ledger ID.
- `UnbillError::InvalidInvitation` — expired or already-used join token.
- `UnbillError::NotAuthorized` — peer attempted to sync a ledger they're not a member of.
- `UnbillError::Storage(StorageError::Sqlite(...))` — I/O or schema errors.

Callers (CLI, Tauri) are expected to map these to user-facing messages.

## 5. Dependencies

| Crate | Why |
|-------|-----|
| `automerge` | CRDT core |
| `autosurgeon` | ergonomic struct ↔ Automerge mapping |
| `iroh` | P2P transport with built-in NAT traversal |
| `tokio` | async runtime |
| `rusqlite` | SQLite storage |
| `ulid` | ID generation |
| `dashmap` | concurrent ledger map in `UnbillService` |
| `dirs` | platform data directory |
| `serde` / `ciborium` | message serialization |

## 6. Testing strategy

- Unit tests in each module, using `InMemoryStore` for storage-dependent tests.
- CRDT convergence tests with `proptest` — arbitrary operation interleavings, cross-merge, assert equal state.
- Sync protocol tests over in-process channels (no real Iroh endpoints).
- Settlement property tests: total owed = total paid, ≤ n-1 transactions.

## 7. Open questions

- **sqlx vs rusqlite**: deferred to `storage/DESIGN.md` (M2 decision point).
- **Lazy vs eager ledger loading at startup**: see project-level §14.
- **Sub-module DESIGN.md files**: each of `model/`, `doc/`, `storage/`, `net/`, `service/`, `settlement/` should get its own brief `DESIGN.md` before substantive implementation begins.
