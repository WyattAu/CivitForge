# ADR-0005: i18n via type-safe `Key` enum + Leptos context

- Status: accepted
- Date: 2026-09-06
- Deciders: Wyatt

## Context

The original i18n used a `thread_local!` locale + stringly-typed `t("nav.home")`
lookups. Leptos cannot observe `thread_local` state, so 139 call sites never
re-rendered on locale change; per-call-site `let _l = signal.get()` hacks were
unmaintainable and a "stuck in Chinese" bug shipped.

## Decision

- `Locale` enum (En/Zh/Ja/Ko) with `from_storage_value` → system-preference → `En`
  fallback chain; persisted to `civitforge_locale`.
- `Key` enum with **exhaustive** per-locale translation matches; `tr(locale)`
  returns `&'static str` (zero allocation, O(1) jump table).
- `I18nContext` (Leptos `StoredValue<Locale>`, `Copy`) provided once in `App`;
  components call `i18n.tr(Key::Variant)` — reactivity is structural, not hacked.
- Legacy `t(&str)` kept only as a migration shim.

## Consequences

- Missing translations are a compile-time impossibility for supported locales;
  untranslated keys fall back to English at the match level (never empty, never panic).
- Adding a locale = adding one enum variant + one match arm block (compiler
  enforces exhaustiveness everywhere).
- HFT/defense alignment: no allocation on the render path, bounded key domain,
  deterministic fallbacks.
