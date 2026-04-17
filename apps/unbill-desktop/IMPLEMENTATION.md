# unbill-desktop — Implementation Notes

## Dependencies

| Package | Why |
|---------|-----|
| `@tauri-apps/api` | Bridge to Rust backend (`invoke`, event listeners) |
| `@tanstack/react-query` | Server-state cache and data fetching |
| `tailwindcss` | Utility-first styling |
| `shadcn/ui` | Component primitives (added as needed at M5) |

## Source structure

- `src/main.tsx` — React root and query client provider.
- `src/App.tsx` — Top-level layout and routing.
- `src/hooks/useUnbillEvent.ts` — Subscribes to `unbill:*` Tauri events and invalidates relevant query caches.
- `src/api/unbill.ts` — Typed wrappers around `invoke`. The only place raw Tauri calls are made.
- `src/components/` — Feature components: LedgerList, LedgerView, AddBillForm, SettlementView.
- `src/lib/format.ts` — Currency formatting and date display helpers.

## Testing strategy

Component tests with Vitest + Testing Library for complex UI logic (e.g., the share-weight picker in AddBillForm). Full-stack manual testing at M5 for Tauri IPC and event flows.
