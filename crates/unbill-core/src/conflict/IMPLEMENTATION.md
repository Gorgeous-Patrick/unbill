# conflict — Implementation

The module is pure and takes no I/O. Its only input is the full bill list from a `LedgerDoc`; its output is a `Vec<ConflictGroup>`.

## Structure

`detect` builds a Union-Find over every bill ID in the full bill list, unions each bill with the IDs in its `prev` list, then groups the effective bills by root. Groups with one member are discarded; groups with two or more are returned as `ConflictGroup` values.

## Types

- `ConflictGroup` — a `Vec<Bill>` of effective bills that share a Union-Find root; always has at least two members.

## Testing

Tests assert the following behaviors:

- A ledger with no amendments produces no conflict groups.
- A linear amendment chain (A → B → C) produces no conflict groups.
- Two independent amendments of the same bill produce one conflict group containing both.
- Merging a conflict group (creating D with `prev = [B, C]`) removes the conflict.
- A multi-bill `prev` that supersedes a chain produces no spurious conflicts.
- Results are deterministic regardless of bill insertion order.
