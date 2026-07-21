pub mod en;
pub mod ja;
pub mod ko;
pub mod zh;

use leptos::prelude::*;

thread_local! {
    static CURRENT_LOCALE: std::cell::RefCell<String> = std::cell::RefCell::new(String::from("en"));
}

/// Reactive locale signal — components must use this to trigger re-renders.
pub fn locale_signal() -> (ReadSignal<String>, WriteSignal<String>) {
    signal(get_locale())
}

pub fn set_locale(locale: &str) {
    CURRENT_LOCALE.with(|c| *c.borrow_mut() = locale.to_string());
}

pub fn get_locale() -> String {
    CURRENT_LOCALE.with(|c| c.borrow().clone())
}

/// Non-reactive translation — use only in static contexts.
pub fn t(key: &str) -> String {
    let locale = get_locale();
    match locale.as_str() {
        "zh" => zh::get(key),
        "ja" => ja::get(key),
        "ko" => ko::get(key),
        _ => en::get(key),
    }
}

/// Reactive translation — returns a closure that re-renders when locale changes.
pub fn tr(key: &'static str, locale_sig: ReadSignal<String>) -> impl Fn() -> String {
    move || {
        let locale = locale_sig.get();
        match locale.as_str() {
            "zh" => zh::get(key),
            "ja" => ja::get(key),
            "ko" => ko::get(key),
            _ => en::get(key),
        }
    }
}

pub fn init_locale_from_storage() {
    #[cfg(feature = "csr")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(Some(locale)) = storage.get_item("civitforge_locale")
            && !locale.is_empty()
        {
            set_locale(&locale);
        }
    }
}

pub fn save_locale_to_storage(locale: &str) {
    #[cfg(feature = "csr")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            let _ = storage.set_item("civitforge_locale", locale);
        }
    }
    set_locale(locale);
}

pub const LOCALES: &[(&str, &str)] = &[
    ("en", "English"),
    ("zh", "中文"),
    ("ja", "日本語"),
    ("ko", "한국어"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_locale_is_en() {
        set_locale("en");
        assert_eq!(get_locale(), "en");
    }

    #[test]
    fn test_set_locale() {
        set_locale("zh");
        assert_eq!(get_locale(), "zh");
    }

    #[test]
    fn test_t_returns_en_by_default() {
        set_locale("en");
        assert_eq!(t("app.name"), "CivitForge");
    }

    #[test]
    fn test_t_zh() {
        set_locale("zh");
        assert_eq!(t("app.name"), "CivitForge");
        assert_eq!(t("nav.home"), "首页");
    }

    #[test]
    fn test_t_ja() {
        set_locale("ja");
        assert_eq!(t("nav.home"), "ホーム");
    }

    #[test]
    fn test_t_ko() {
        set_locale("ko");
        assert_eq!(t("nav.home"), "홈");
    }

    #[test]
    fn test_t_fallback_to_en() {
        set_locale("en");
        assert_eq!(t("nonexistent.key"), "nonexistent.key");
    }
}
