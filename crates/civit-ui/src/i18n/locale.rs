/// Locale state machine — finite set of supported languages.
///
/// Invariant: as_str() returns a valid BCP-47 subtag.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    En,
    Zh,
    Ja,
    Ko,
}

impl Locale {
    /// BCP-47 subtag. Provable: never empty, never contains spaces.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
            Self::Ja => "ja",
            Self::Ko => "ko",
        }
    }

    /// Human-readable name in the locale's own script.
    pub const fn native_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Zh => "中文",
            Self::Ja => "日本語",
            Self::Ko => "한국어",
        }
    }

    /// All supported locales for iteration.
    pub const ALL: &'static [Locale] = &[Locale::En, Locale::Zh, Locale::Ja, Locale::Ko];

    /// Parse from localStorage string. Corrupted/missing → system → En.
    pub fn from_storage_value(s: Option<&str>) -> Self {
        match s {
            Some("en") => Self::En,
            Some("zh") => Self::Zh,
            Some("ja") => Self::Ja,
            Some("ko") => Self::Ko,
            _ => Self::from_system_preference(),
        }
    }

    /// Parse from navigator.language. Falls back to En.
    fn from_system_preference() -> Self {
        #[cfg(feature = "csr")]
        {
            if let Some(window) = web_sys::window() {
                let lang = window.navigator().language().unwrap_or_default();
                if lang.starts_with("zh") {
                    return Self::Zh;
                }
                if lang.starts_with("ja") {
                    return Self::Ja;
                }
                if lang.starts_with("ko") {
                    return Self::Ko;
                }
            }
        }
        Self::En
    }

    /// Persist to localStorage.
    pub fn persist(self) {
        #[cfg(feature = "csr")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("civitforge_locale", self.as_str());
                }
            }
        }
    }

    /// Full change: set, persist. Returns new locale.
    pub fn switch_and_persist(new_locale: Self) -> Self {
        new_locale.persist();
        new_locale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_is_bcp47() {
        for locale in Locale::ALL {
            assert!(!locale.as_str().is_empty());
            assert!(!locale.as_str().contains(' '));
        }
    }

    #[test]
    fn all_locales_have_native_names() {
        for locale in Locale::ALL {
            assert!(!locale.native_name().is_empty());
        }
    }

    #[test]
    fn from_storage_value_parse() {
        assert_eq!(Locale::from_storage_value(Some("en")), Locale::En);
        assert_eq!(Locale::from_storage_value(Some("zh")), Locale::Zh);
        assert_eq!(Locale::from_storage_value(Some("ja")), Locale::Ja);
        assert_eq!(Locale::from_storage_value(Some("ko")), Locale::Ko);
        assert_eq!(Locale::from_storage_value(Some("fr")), Locale::En); // unknown → En
        assert_eq!(Locale::from_storage_value(None), Locale::En);
    }
}
