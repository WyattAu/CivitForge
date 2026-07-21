pub mod keys;
pub mod locale;

pub use keys::Key;
pub use locale::Locale;

use leptos::prelude::*;

/// Context holding the reactive locale state.
/// Single source of truth. All translations read from here.
#[derive(Clone, Copy)]
pub struct I18nContext {
    locale: StoredValue<Locale>,
}

impl I18nContext {
    /// Get current locale. Reactive: re-renders caller when locale changes.
    pub fn locale(&self) -> Locale {
        self.locale.get_value()
    }

    /// Translate a key using the current locale. Zero allocation.
    pub fn tr(&self, key: Key) -> &'static str {
        key.tr(self.locale.get_value())
    }

    /// Switch locale, persist to localStorage, trigger re-render of all
    /// components that read from this context.
    pub fn set_locale(&self, new_locale: Locale) {
        Locale::switch_and_persist(new_locale);
        self.locale.set_value(new_locale);
    }
}

/// Provide i18n context. Call once in App component.
pub fn provide_i18n() -> I18nContext {
    let initial = get_stored_locale()
        .map(|s| Locale::from_storage_value(Some(s.as_str())))
        .unwrap_or(Locale::En);
    let locale = StoredValue::new(initial);
    let ctx = I18nContext { locale };
    provide_context(ctx.clone());
    ctx
}

/// Get the i18n context from anywhere in the component tree.
pub fn use_i18n() -> I18nContext {
    expect_context::<I18nContext>()
}

/// Read locale from localStorage. Returns None if unavailable.
fn get_stored_locale() -> Option<String> {
    #[cfg(feature = "csr")]
    {
        web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .and_then(|s| s.get_item("civitforge_locale").ok())
            .flatten()
            .filter(|s| !s.is_empty())
    }
    #[cfg(not(feature = "csr"))]
    {
        None
    }
}

/// Legacy compatibility: translate a dot-key string using current locale.
/// Prefer Key enum in new code.
pub fn t(key: &str) -> String {
    if let Some(k) = Key::from_str_key(key) {
        let locale = get_stored_locale()
            .map(|s| Locale::from_storage_value(Some(s.as_str())))
            .unwrap_or(Locale::En);
        k.tr(locale).to_string()
    } else {
        key.to_string()
    }
}

/// All supported locales for UI iteration.
pub const LOCALES: &[(Locale, &str)] = &[
    (Locale::En, "English"),
    (Locale::Zh, "中文"),
    (Locale::Ja, "日本語"),
    (Locale::Ko, "한국어"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_fallback_to_key_string() {
        assert_eq!(t("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn t_translates_known_keys() {
        // Default locale (no localStorage) → English
        assert_eq!(t("nav.home"), "Home");
        assert_eq!(t("auth.sign_in"), "Sign In");
    }
}
