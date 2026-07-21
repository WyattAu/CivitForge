/// Theme state machine — 2 states, deterministic transitions.
///
/// Invariant: self.class_name() == "dark" ⟺ self == Theme::Dark
/// Invariant: self.class_name().is_empty() ⟺ self == Theme::Light
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    /// CSS class applied to `<html>`. Provable: empty ⟺ Light, "dark" ⟺ Dark.
    pub const fn class_name(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "",
        }
    }

    /// Toggle: Dark → Light, Light → Dark. Deterministic, no allocation.
    pub const fn toggle(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    /// Parse from localStorage string. Corrupted/missing → system → Light.
    /// Defense-grade: never panics, never returns invalid state.
    pub fn from_storage_value(s: Option<&str>) -> Self {
        match s {
            Some("dark") => Self::Dark,
            Some("light") => Self::Light,
            _ => Self::from_system_preference(),
        }
    }

    /// Read system preference via matchMedia.
    /// Fallback: Light (most conservative default).
    fn from_system_preference() -> Self {
        #[cfg(feature = "csr")]
        {
            if let Some(window) = web_sys::window() {
                let media = window.match_media("(prefers-color-scheme: dark)");
                if let Ok(Some(m)) = media {
                    if m.matches() {
                        return Self::Dark;
                    }
                }
            }
        }
        Self::Light
    }

    /// Persist to localStorage.
    pub fn persist(self) {
        #[cfg(feature = "csr")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("civit-theme", self.class_name());
                }
            }
        }
    }

    /// Apply to DOM: toggle the "dark" class on `<html>`.
    pub fn apply_to_dom(self) {
        #[cfg(feature = "csr")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Some(html) = doc.document_element() {
                        let class_list = html.class_list();
                        if self == Self::Dark {
                            let _ = class_list.add_1("dark");
                        } else {
                            let _ = class_list.remove_1("dark");
                        }
                    }
                }
            }
        }
    }

    /// Full toggle: change state, persist, apply to DOM. Single function.
    pub fn toggle_and_persist(current: Self) -> Self {
        let next = current.toggle();
        next.persist();
        next.apply_to_dom();
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_is_involution() {
        assert_eq!(Theme::Dark.toggle().toggle(), Theme::Dark);
        assert_eq!(Theme::Light.toggle().toggle(), Theme::Light);
    }

    #[test]
    fn toggle_produces_opposite() {
        assert_eq!(Theme::Dark.toggle(), Theme::Light);
        assert_eq!(Theme::Light.toggle(), Theme::Dark);
    }

    #[test]
    fn class_name_invariant() {
        assert!(Theme::Light.class_name().is_empty());
        assert_eq!(Theme::Dark.class_name(), "dark");
    }

    #[test]
    fn from_storage_value_parse() {
        assert_eq!(Theme::from_storage_value(Some("dark")), Theme::Dark);
        assert_eq!(Theme::from_storage_value(Some("light")), Theme::Light);
        assert_eq!(Theme::from_storage_value(Some("invalid")), Theme::Light);
        assert_eq!(Theme::from_storage_value(None), Theme::Light);
    }
}
