# unbill-i18n Design

## Purpose

`unbill-i18n` provides a Rust-native internationalization contract for Unbill. It centralizes user-facing strings behind stable keys and locale-aware translation lookup.

## Contract

- Strings are addressed by typed keys, not ad-hoc string literals.
- Callers request a translation by `(locale, key)`.
- If a locale is unsupported, translation falls back to English.
- If a key is not translated in a selected locale, translation falls back to English.
- If a key is missing globally, the crate returns the key identifier for debugging visibility.

## Locale model

The crate supports locale tags as BCP-47-like short tags (`en`, `zh-CN`, etc.) represented as a typed enum for compile-time safety.

## Scope

This crate owns:

- Translation keys
- Locale selection and fallback behavior
- The translation catalog bundled in Rust source

This crate does not own:

- Runtime locale negotiation from HTTP headers
- Persistent user locale preference storage
- Rich ICU-style message formatting
