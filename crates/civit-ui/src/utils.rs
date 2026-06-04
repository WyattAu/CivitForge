use crate::components::BadgeColor;

#[cfg(feature = "csr")]
use wasm_bindgen::JsCast;

pub fn relative_time(timestamp: &str) -> String {
    #[cfg(feature = "csr")]
    {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(dt);
            if diff.num_seconds() < 60 {
                return "just now".to_string();
            } else if diff.num_minutes() < 60 {
                return format!("{}m ago", diff.num_minutes());
            } else if diff.num_hours() < 24 {
                return format!("{}h ago", diff.num_hours());
            } else if diff.num_days() < 30 {
                return format!("{}d ago", diff.num_days());
            } else {
                return dt.format("%b %d, %Y").to_string();
            }
        }
    }
    timestamp.to_string()
}

pub fn truncate_uuid(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

pub fn get_input_value(name: &str) -> String {
    #[cfg(feature = "csr")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return String::new(),
        };
        let doc = match window.document() {
            Some(d) => d,
            None => return String::new(),
        };
        let el = match doc.get_element_by_id(name) {
            Some(el) => el,
            None => return String::new(),
        };
        let tag = el.tag_name().to_lowercase();
        if tag == "textarea" {
            match el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                Ok(ta) => return ta.value(),
                Err(_) => return String::new(),
            }
        }
        match el.dyn_into::<web_sys::HtmlInputElement>() {
            Ok(input) => input.value(),
            Err(_) => String::new(),
        }
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = name;
        String::new()
    }
}

pub fn status_badge_color(state: &str) -> BadgeColor {
    match state {
        "open" => BadgeColor::Success,
        "in_progress" => BadgeColor::Info,
        "closed" => BadgeColor::Neutral,
        _ => BadgeColor::Neutral,
    }
}

pub fn status_label(state: &str) -> String {
    match state {
        "in_progress" => "In Progress".to_string(),
        s => {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
    }
}

pub fn sanitize_error(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '.' || *c == '-')
        .collect::<String>()
        .trim()
        .to_string()
}
