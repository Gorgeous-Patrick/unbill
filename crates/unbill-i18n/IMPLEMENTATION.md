# unbill-i18n Implementation

## Structure

- `src/lib.rs` contains:
  - `Locale` enum (supported locales)
  - `MessageKey` enum (supported translation identifiers)
  - `I18n` translation catalog type

## Translation storage

Translations are implemented in Rust through match-based lookup functions keyed by `(Locale, MessageKey)`.

## Fallback algorithm

1. Attempt translation for requested locale.
2. Fall back to English for missing locale/key mapping.
3. Fall back to key name string for globally missing entries.

## Testing

`src/lib.rs` includes unit tests that verify:

- English translations resolve expected values.
- Chinese translations resolve expected values.
- Unsupported locale parsing falls back to English by caller contract.
- Missing localized keys fall back to English.
