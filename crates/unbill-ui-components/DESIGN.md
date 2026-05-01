# unbill-ui-components

Reusable Leptos UI components for unbill frontends.

## Contract

- Components are pure Leptos — no Tauri IPC, no HTTP, no unbill-core imports.
- All operations that touch external state (loading data, submitting forms, navigating) are passed as `Callback` props. The caller wires them to whatever backend is in use (Tauri, mock, test stub, etc.).
- Components own only local UI state: expanded/collapsed, hover, validation feedback, pending spinner.
- Data types used in props and events are defined in this crate. They are plain structs — no domain-model imports.

## Structure

```
src/
  lib.rs          re-exports all public components and types
  button.rs       ActionButton, IconButton
  input.rs        TextInput, CurrencyInput, AmountInput
  layout.rs       Page, SafeAreaContainer, Sheet (bottom drawer)
  bill.rs         BillRow, BillList, BillForm
  user.rs         UserRow, UserList, UserAvatar
  ledger.rs       LedgerRow, LedgerList
  settlement.rs   SettlementRow, SettlementList
```

## Rules

- A component that submits a form calls `on_submit: Callback<FormData>` and returns to idle state; the caller decides what happens next.
- A component that loads a list accepts `items: Signal<Vec<T>>` and an optional `on_refresh: Callback<()>`.
- Error and loading states are expressed through the signal values, not through separate props.
