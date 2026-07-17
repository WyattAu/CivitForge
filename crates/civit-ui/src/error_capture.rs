//! Global error capture for the CivitForge frontend.
//! Intercepts console errors, unhandled exceptions, and Leptos errors.

use leptos::wasm_bindgen::prelude::*;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct CapturedError {
    pub timestamp: String,
    pub source: String,
    pub message: String,
    pub url: String,
    pub stack: Option<String>,
}

static ERROR_STORE: std::sync::OnceLock<RwLock<Vec<CapturedError>>> = std::sync::OnceLock::new();

fn store() -> &'static RwLock<Vec<CapturedError>> {
    ERROR_STORE.get_or_init(|| RwLock::new(Vec::new()))
}

pub fn capture_error(source: &str, message: String, stack: Option<String>) {
    let url = get_url();
    let ts = if cfg!(target_arch = "wasm32") {
        js_sys::Date::new_0()
            .to_iso_string()
            .as_string()
            .unwrap_or_default()
    } else {
        format!("{:?}", std::time::SystemTime::now())
    };
    let entry = CapturedError {
        timestamp: ts,
        source: source.to_string(),
        message,
        url,
        stack,
    };
    if let Ok(mut s) = store().write() {
        s.push(entry);
        if s.len() > 500 {
            let n = s.len() - 500;
            s.drain(0..n);
        }
    }
}

pub fn get_captured_errors() -> Vec<CapturedError> {
    store().read().map(|s| s.clone()).unwrap_or_default()
}

pub fn clear_captured_errors() {
    if let Ok(mut s) = store().write() {
        s.clear();
    }
}

pub fn has_errors() -> bool {
    store().read().map(|s| !s.is_empty()).unwrap_or(false)
}

pub fn error_count() -> usize {
    store().read().map(|s| s.len()).unwrap_or(0)
}

fn get_url() -> String {
    if cfg!(target_arch = "wasm32") {
        let window = web_sys::window().expect("browser window available");
        window.location().href().unwrap_or_default()
    } else {
        "test://non-wasm".to_string()
    }
}

const ERROR_LISTENER_JS: &str = r#"
(function() {
    window.__civitforgeErrors = [];
    
    window.onerror = function(msg, url, line, col, error) {
        window.__civitforgeErrors.push({
            source: 'unhandled',
            message: String(msg),
            url: url || '',
            stack: error ? (error.stack || '') : '',
            timestamp: new Date().toISOString()
        });
        return false;
    };
    
    window.addEventListener('unhandledrejection', function(event) {
        window.__civitforgeErrors.push({
            source: 'unhandled_promise',
            message: String(event.reason),
            url: window.location.href,
            stack: event.reason && event.reason.stack ? event.reason.stack : '',
            timestamp: new Date().toISOString()
        });
    });
    
    var origConsoleError = console.error;
    console.error = function() {
        var args = Array.prototype.slice.call(arguments);
        origConsoleError.apply(console, args);
        window.__civitforgeErrors.push({
            source: 'console',
            message: args.map(function(a) { return typeof a === 'object' ? JSON.stringify(a) : String(a); }).join(' '),
            url: window.location.href,
            stack: '',
            timestamp: new Date().toISOString()
        });
    };
    
    var origConsoleWarn = console.warn;
    console.warn = function() {
        var args = Array.prototype.slice.call(arguments);
        origConsoleWarn.apply(console, args);
        window.__civitforgeErrors.push({
            source: 'console_warn',
            message: args.map(function(a) { return typeof a === 'object' ? JSON.stringify(a) : String(a); }).join(' '),
            url: window.location.href,
            stack: '',
            timestamp: new Date().toISOString()
        });
    };
})();
"#;

pub fn install_global_error_listeners() {
    if cfg!(target_arch = "wasm32") {
        let _ = js_sys::eval(ERROR_LISTENER_JS);
    }
}

/// Synchronize JS-captured errors into the Rust error store
pub fn sync_js_errors() {
    use leptos::wasm_bindgen::JsCast;
    let window = web_sys::window().expect("browser window available");
    let errors = js_sys::Reflect::get(&window, &JsValue::from_str("__civitforgeErrors"))
        .ok()
        .and_then(|v| {
            if v.is_undefined() || v.is_null() {
                None
            } else {
                Some(v)
            }
        });

    if let Some(arr) = errors {
        if let Ok(arr) = arr.dyn_into::<js_sys::Array>() {
            for i in 0..arr.length() {
                if let Ok(obj) = arr.get(i).dyn_into::<js_sys::Object>() {
                    let source = js_sys::Reflect::get(&obj, &"source".into())
                        .ok()
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    let message = js_sys::Reflect::get(&obj, &"message".into())
                        .ok()
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    let stack = js_sys::Reflect::get(&obj, &"stack".into())
                        .ok()
                        .and_then(|v| v.as_string())
                        .filter(|s| !s.is_empty());
                    capture_error(&source, message, stack);
                }
            }
        }
        // Clear the JS array
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("__civitforgeErrors"),
            &js_sys::Array::new(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_error_capture_sequential() {
        // Tests must run sequentially because they share a static OnceLock store
        clear_captured_errors();
        assert!(!has_errors());

        // Test capture and retrieve
        capture_error("test", "error message".to_string(), None);
        let errors = get_captured_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].source, "test");
        assert_eq!(errors[0].message, "error message");

        // Test clear
        clear_captured_errors();
        assert!(!has_errors());

        // Test count
        assert_eq!(error_count(), 0);
        capture_error("test", "a".to_string(), None);
        capture_error("test", "b".to_string(), None);
        assert_eq!(error_count(), 2);

        // Test max 500
        clear_captured_errors();
        for i in 0..600u32 {
            capture_error("test", format!("error {i}"), None);
        }
        assert_eq!(error_count(), 500);
    }
}
