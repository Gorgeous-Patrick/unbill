# unbill-tui

A full-featured interactive terminal frontend for unbill. It presents the same capabilities as the CLI in a persistent, keyboard-driven interface without requiring a graphical desktop environment.

## Layout

The screen is divided into three vertical panes that are always visible.

```
┌────────────────┬───────────────────────┬──────────────────────────┐
│ Ledgers        │ Bills                 │ Detail                   │
│                │                       │                          │
│ > Household    │ > Dinner      $30.00  │                          │
│   Trip         │   Lunch       $12.50  │                          │
│   Road trip    │   Coffee       $5.00  │                          │
│                │                       │                          │
│                │                       │                          │
├────────────────┴───────────────────────┴──────────────────────────┤
│ [j/k] move  [h/l] pane  [a]dd  [e]dit  [u]sers  [s]ettle  [q]uit │
└───────────────────────────────────────────────────────────────────┘
```

- **Left pane — Ledger list.** All ledgers on this device. Moving the cursor through this list immediately updates the bill pane to show bills for the cursor-focused ledger.
- **Middle pane — Bill list.** Effective bills in the cursor-focused ledger, shown with description and amount.
- **Right pane — Detail.** Reserved for a future iteration; currently empty.

## Focus and Navigation

One pane is active at a time. The active pane has a highlighted border.

| Key | Action |
|-----|--------|
| `h` | Move focus to left pane |
| `l` | Move focus to right pane |
| `Tab` | Move focus right (wraps) |
| `Shift+Tab` | Move focus left (wraps) |
| `Enter` | Move focus right (same as `l`) |
| `j` / `k` | Move cursor down / up within the focused pane |
| `g` | Jump to first item |
| `G` | Jump to last item |

## Actions

Actions are context-sensitive. The status bar shows only the keys valid for the currently focused pane.

| Key | Ledger pane | Bill pane |
|-----|-------------|-----------|
| `a` | Create ledger | Add bill |
| `e` | — | Amend selected bill |
| `d` | Delete ledger (confirmation required) | — |
| `u` | Manage users in ledger | — |
| `s` | Open settlement for a local user | — |
| `S` | Trigger manual sync | — |
| `i` | Open invite / join modal | — |
| `q` | Quit | Quit |
| `Esc` | Close modal / cancel | Close modal / cancel |

Bills are append-only and cannot be deleted.

## Status Bar

The status bar at the bottom of the screen has two parts:

- **Left** — context-sensitive key hints for the currently focused pane.
- **Right** — sync status indicator: idle, syncing, or last error.

The hints update immediately when focus changes.

## Modals

Actions that require input open a modal overlay. The rest of the screen dims. Within a modal, `Tab` / `Shift+Tab` move between fields, `Enter` confirms, `Esc` cancels.

### Create ledger
Two fields: name and ISO 4217 currency code.

### Add bill
Fields: description, amount (decimal), payer (chosen from ledger users), share users (multi-select from ledger users with equal shares).

### Amend bill
Same fields as add bill, pre-filled from the selected bill. The `prev` link to the selected bill is set automatically on confirm.

### User management
Shows the current users in the focused ledger. A sub-action allows adding a local device user to the ledger by selecting from the device's saved local users. New local users can also be created here.

### Settlement
First presents a list of saved local users on this device. After one is chosen, shows the net transactions for that user across all ledgers.

### Invite / join
Two tabs within the modal:
- **Invite** — generates and displays an `unbill://join/...` URL for the focused ledger.
- **Join** — accepts a pasted `unbill://join/...` URL to join a ledger hosted by another device.

### Confirm delete
A yes/no prompt shown before deleting a ledger.

## Sync

A background task subscribes to `ServiceEvent`s from the service and refreshes the visible panes when a `LedgerUpdated` event arrives. Manual sync is triggered with `S`, which opens a prompt for a peer `NodeId` and then dials that peer.

## Empty States

- No ledgers: left pane shows a dim `no ledgers — press [a] to create one`.
- No bills: middle pane shows a dim `no bills — press [a] to add one`.
- No local users (for settlement): settlement modal shows `no saved users — create one first`.

## Principles

- No business logic lives in the TUI. All mutations go through `UnbillService`.
- The TUI treats the service as the source of truth and re-reads it after every mutation.
- The bill pane always reflects the ledger under the cursor, not the last confirmed selection. Navigation is immediate.
- Keyboard shortcuts follow vim conventions where there is a natural mapping; other keys are mnemonic.
- Errors from the service are shown in the status bar and never crash the TUI.
