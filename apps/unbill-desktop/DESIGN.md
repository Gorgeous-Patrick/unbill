# unbill-desktop — Design Document

> Status: Stub (fill before M5)

## 1. Purpose

The React frontend served inside the Tauri webview. Responsible for all user-visible UI. Communicates with the Rust backend exclusively via `@tauri-apps/api` `invoke` calls and event listeners.

## 2. Public API sketch

No external API. Internal structure:

```
src/
├── main.tsx            # React root, QueryClientProvider
├── App.tsx             # top-level route / layout
├── hooks/
│   └── useUnbillEvent.ts   # subscribe to unbill:* Tauri events, invalidate TanStack Query caches
├── api/
│   └── unbill.ts       # typed wrappers around invoke("command_name")
├── components/
│   ├── LedgerList/
│   ├── LedgerView/
│   ├── AddBillForm/    # includes share-weight picker (equal / custom weights)
│   └── SettlementView/
└── lib/
    └── format.ts       # currency formatting, date helpers
```

## 3. Invariants

- The frontend never computes business logic (settlement, amendment projection). It displays what the backend returns.
- All backend calls go through `src/api/unbill.ts`; no raw `invoke` elsewhere.
- IDs are treated as opaque strings on the JS side (ULID format, but the frontend does not parse them).

## 4. Failure modes

- Failed `invoke` calls surface as toast notifications.
- Stale data is handled by TanStack Query's cache invalidation on `ServiceEvent`s.

## 5. Dependencies

| Package | Why |
|---------|-----|
| `@tauri-apps/api` | bridge to Rust backend |
| `@tanstack/react-query` | server-state cache and data fetching |
| `tailwindcss` | utility-first styling |
| shadcn/ui | component primitives (added as needed at M5) |

## 6. Testing strategy

- Manual testing in M5.
- Component tests with Vitest + Testing Library for complex UI logic (e.g., `AddBillForm` share-weight picker).

## 7. Open questions

- Routing: single-page with React state, or React Router? Decide at M5 based on actual screen count.
- Theming: system dark/light mode support via Tailwind's `dark:` variants.
- i18n: deferred post-M5.
