#![cfg(all(target_arch = "wasm32", feature = "csr"))]
#![forbid(unsafe_code)]

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::*;

wasm_bindgen_test_configure!(run_in_browser);

fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

#[wasm_bindgen_test]
fn test_create_div_element() {
    let doc = document();
    let div: HtmlDivElement = doc.create_element("div").unwrap().dyn_into().unwrap();
    assert_eq!(div.tag_name(), "DIV");
}

#[wasm_bindgen_test]
fn test_create_paragraph_element() {
    let doc = document();
    let p: HtmlParagraphElement = doc.create_element("p").unwrap().dyn_into().unwrap();
    assert_eq!(p.tag_name(), "P");
}

#[wasm_bindgen_test]
fn test_create_span_element() {
    let doc = document();
    let span: HtmlSpanElement = doc.create_element("span").unwrap().dyn_into().unwrap();
    assert_eq!(span.tag_name(), "SPAN");
}

#[wasm_bindgen_test]
fn test_create_anchor_element() {
    let doc = document();
    let a: HtmlAnchorElement = doc.create_element("a").unwrap().dyn_into().unwrap();
    assert_eq!(a.tag_name(), "A");
}

#[wasm_bindgen_test]
fn test_create_button_element() {
    let doc = document();
    let btn: HtmlButtonElement = doc.create_element("button").unwrap().dyn_into().unwrap();
    assert_eq!(btn.tag_name(), "BUTTON");
}

#[wasm_bindgen_test]
fn test_create_input_element() {
    let doc = document();
    let input: HtmlInputElement = doc.create_element("input").unwrap().dyn_into().unwrap();
    assert_eq!(input.tag_name(), "INPUT");
}

#[wasm_bindgen_test]
fn test_create_form_element() {
    let doc = document();
    let form: HtmlFormElement = doc.create_element("form").unwrap().dyn_into().unwrap();
    assert_eq!(form.tag_name(), "FORM");
}

#[wasm_bindgen_test]
fn test_create_text_node() {
    let doc = document();
    let text: Text = doc.create_text_node("hello").dyn_into().unwrap();
    assert_eq!(text.text_content(), Some("hello".to_string()));
}

#[wasm_bindgen_test]
fn test_set_text_content() {
    let doc = document();
    let div: HtmlDivElement = doc.create_element("div").unwrap().dyn_into().unwrap();
    div.set_text_content(Some("hello world"));
    assert_eq!(div.text_content(), Some("hello world".to_string()));
}

#[wasm_bindgen_test]
fn test_inner_text() {
    let doc = document();
    let body = doc.body().unwrap();
    let div: HtmlDivElement = doc.create_element("div").unwrap().dyn_into().unwrap();
    div.set_text_content(Some("inner text"));
    body.append_child(&div).unwrap();
    assert_eq!(div.inner_text().unwrap_or_default(), "inner text");
    body.remove_child(&div).ok();
}

#[wasm_bindgen_test]
fn test_class_list_add() {
    let doc = document();
    let div: HtmlDivElement = doc.create_element("div").unwrap().dyn_into().unwrap();
    div.class_list().add_1("container").unwrap();
    assert!(div.class_list().contains("container"));
}

#[wasm_bindgen_test]
fn test_class_list_add_multiple() {
    let doc = document();
    let div: HtmlDivElement = doc.create_element("div").unwrap().dyn_into().unwrap();
    div.class_list().add_2("flex", "items-center").unwrap();
    assert!(div.class_list().contains("flex"));
    assert!(div.class_list().contains("items-center"));
}

#[wasm_bindgen_test]
fn test_class_list_remove() {
    let doc = document();
    let div: HtmlDivElement = doc.create_element("div").unwrap().dyn_into().unwrap();
    div.class_list().add_1("removable").unwrap();
    assert!(div.class_list().contains("removable"));
    div.class_list().remove_1("removable").unwrap();
    assert!(!div.class_list().contains("removable"));
}

#[wasm_bindgen_test]
fn test_append_child() {
    let doc = document();
    let body = doc.body().unwrap();
    let div: HtmlDivElement = doc.create_element("div").unwrap().dyn_into().unwrap();
    div.set_id("test-append-child");
    body.append_child(&div).unwrap();
    let found = doc.query_selector("#test-append-child").unwrap();
    assert!(found.is_some());
    body.remove_child(&div).ok();
}

#[wasm_bindgen_test]
fn test_remove_child() {
    let doc = document();
    let body = doc.body().unwrap();
    let div: HtmlDivElement = doc.create_element("div").unwrap().dyn_into().unwrap();
    div.set_id("test-remove-child");
    body.append_child(&div).unwrap();
    assert!(doc.query_selector("#test-remove-child").unwrap().is_some());
    body.remove_child(&div).unwrap();
    assert!(doc.query_selector("#test-remove-child").unwrap().is_none());
}

#[wasm_bindgen_test]
fn test_query_selector_all() {
    let doc = document();
    let body = doc.body().unwrap();
    let p1: HtmlParagraphElement = doc.create_element("p").unwrap().dyn_into().unwrap();
    let p2: HtmlParagraphElement = doc.create_element("p").unwrap().dyn_into().unwrap();
    let p3: HtmlParagraphElement = doc.create_element("p").unwrap().dyn_into().unwrap();
    p1.set_class("test-para");
    p2.set_class("test-para");
    p3.set_class("other-para");
    body.append_child(&p1).unwrap();
    body.append_child(&p2).unwrap();
    body.append_child(&p3).unwrap();
    let nodes = doc.query_selector_all(".test-para").unwrap();
    assert_eq!(nodes.length(), 2);
    body.remove_child(&p1).ok();
    body.remove_child(&p2).ok();
    body.remove_child(&p3).ok();
}

#[wasm_bindgen_test]
fn test_element_styling() {
    let doc = document();
    let div: HtmlDivElement = doc.create_element("div").unwrap().dyn_into().unwrap();
    div.style().set_property("color", "red").unwrap();
    assert_eq!(
        div.style().get_property_value("color").unwrap_or_default(),
        "red"
    );
    div.style().set_property("display", "flex").unwrap();
    assert_eq!(
        div.style()
            .get_property_value("display")
            .unwrap_or_default(),
        "flex"
    );
}

#[wasm_bindgen_test]
fn test_input_value_get_set() {
    let doc = document();
    let input: HtmlInputElement = doc.create_element("input").unwrap().dyn_into().unwrap();
    input.set_value("test value");
    assert_eq!(input.value(), "test value");
    input.set_value("");
    assert_eq!(input.value(), "");
    input.set_value("updated");
    assert_eq!(input.value(), "updated");
}

#[wasm_bindgen_test]
fn test_form_element_action() {
    let doc = document();
    let form: HtmlFormElement = doc.create_element("form").unwrap().dyn_into().unwrap();
    form.set_action("/submit");
    assert!(form.action().ends_with("/submit"));
}

#[wasm_bindgen_test]
fn test_form_element_method() {
    let doc = document();
    let form: HtmlFormElement = doc.create_element("form").unwrap().dyn_into().unwrap();
    form.set_method("POST");
    assert_eq!(form.method(), "POST");
}

#[wasm_bindgen_test]
fn test_event_listener_attachment() {
    let doc = document();
    let body = doc.body().unwrap();
    let btn: HtmlButtonElement = doc.create_element("button").unwrap().dyn_into().unwrap();
    body.append_child(&btn).unwrap();

    let cb = Closure::new(move |_| {});
    btn.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
        .unwrap();
    cb.forget();
    body.remove_child(&btn).ok();
}

#[wasm_bindgen_test]
fn test_event_dispatch_and_handle() {
    let doc = document();
    let body = doc.body().unwrap();
    let btn: HtmlButtonElement = doc.create_element("button").unwrap().dyn_into().unwrap();
    btn.set_id("dispatch-test-btn");
    body.append_child(&btn).unwrap();

    js_sys::eval("window.__wasmTestFired = false;").unwrap();

    let cb = Closure::new(move |_| {
        js_sys::eval("window.__wasmTestFired = true;").unwrap();
    });
    btn.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
        .unwrap();

    let event = web_sys::Event::new("click").unwrap();
    btn.dispatch_event(&event).unwrap();

    let fired: bool = js_sys::Reflect::get(&web_sys::window().unwrap(), &"__wasmTestFired".into())
        .unwrap()
        .as_bool()
        .unwrap();

    assert!(fired);
    cb.forget();
    body.remove_child(&btn).ok();
}

#[wasm_bindgen_test]
fn test_uuid_generation() {
    let uuid = uuid::Uuid::new_v4();
    assert!(!uuid.is_nil());
    let s = uuid.to_string();
    assert_eq!(s.len(), 36);
    assert_eq!(s.chars().filter(|&c| c == '-').count(), 4);
}

#[wasm_bindgen_test]
fn test_uuid_parse_roundtrip() {
    let uuid = uuid::Uuid::new_v4();
    let s = uuid.to_string();
    let parsed = uuid::Uuid::parse_str(&s).unwrap();
    assert_eq!(uuid, parsed);
}

#[wasm_bindgen_test]
fn test_chrono_utc_now_roundtrip() {
    let now = chrono::Utc::now();
    let serialized = now.to_rfc3339();
    let deserialized: chrono::DateTime<chrono::Utc> = serialized.parse().unwrap();
    assert_eq!(now, deserialized);
}

#[wasm_bindgen_test]
fn test_panic_hook_setup() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen_test]
fn test_serde_json_roundtrip_struct() {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct TestData {
        name: String,
        value: i32,
    }

    let data = TestData {
        name: "test".to_string(),
        value: 42,
    };
    let json = serde_json::to_string(&data).unwrap();
    let parsed: TestData = serde_json::from_str(&json).unwrap();
    assert_eq!(data, parsed);
}

#[wasm_bindgen_test]
fn test_serde_json_null_and_bool() {
    assert_eq!(
        serde_json::to_string(&serde_json::Value::Null).unwrap(),
        "null"
    );
    assert_eq!(serde_json::to_string(&true).unwrap(), "true");
    assert_eq!(serde_json::to_string(&false).unwrap(), "false");
    let val: serde_json::Value = serde_json::from_str("null").unwrap();
    assert!(val.is_null());
}

#[wasm_bindgen_test]
fn test_error_capture_initialization() {
    crate::error_capture::clear_captured_errors();
    assert!(!crate::error_capture::has_errors());

    crate::error_capture::capture_error("wasm_test", "test error".to_string(), None);
    assert!(crate::error_capture::has_errors());

    let errors = crate::error_capture::get_captured_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].source, "wasm_test");
    assert_eq!(errors[0].message, "test error");

    crate::error_capture::clear_captured_errors();
    assert_eq!(crate::error_capture::error_count(), 0);
}

#[wasm_bindgen_test]
fn test_localstorage_write_read() {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();

    storage
        .set_item("civit_test_key", "civit_test_value")
        .unwrap();
    let val = storage.get_item("civit_test_key").unwrap();
    assert_eq!(val, Some("civit_test_value".to_string()));

    storage.remove_item("civit_test_key").unwrap();
    let after = storage.get_item("civit_test_key").unwrap();
    assert!(after.is_none());
}

#[wasm_bindgen_test]
fn test_url_parsing_via_anchor() {
    let doc = document();
    let a: HtmlAnchorElement = doc.create_element("a").unwrap().dyn_into().unwrap();
    a.set_href("https://example.com/path?query=1#hash");

    assert_eq!(a.hostname(), "example.com");
    assert_eq!(a.pathname(), "/path");
    assert_eq!(a.search(), "?query=1");
    assert_eq!(a.hash(), "#hash");
    assert_eq!(a.protocol(), "https:");
}

#[wasm_bindgen_test]
fn test_document_cookie_access() {
    let doc = document();
    let _cookies = doc.cookie().unwrap_or_default();
}

#[wasm_bindgen_test]
fn test_performance_now() {
    let window = web_sys::window().unwrap();
    let perf = window.performance().unwrap();
    let t = perf.now();
    assert!(t >= 0.0);
    let t2 = perf.now();
    assert!(t2 >= t);
}

#[wasm_bindgen_test]
fn test_request_animation_frame() {
    let window = web_sys::window().unwrap();
    let cb = Closure::new(move |_timestamp: f64| {});
    let handle = window
        .request_animation_frame(cb.as_ref().unchecked_ref())
        .unwrap();
    assert!(handle > 0);
    cb.forget();
}

#[wasm_bindgen_test]
fn test_history_push_state() {
    let window = web_sys::window().unwrap();
    let history = window.history().unwrap();
    let state = js_sys::Object::new();
    history
        .push_state_with_url(&JsValue::from(state), &"test".into(), Some("/wasm-test"))
        .unwrap();

    let loc = window.location();
    let path = loc.pathname().unwrap();
    assert_eq!(path, "/wasm-test");

    history.back().unwrap();
}

#[wasm_bindgen_test]
fn test_window_location_href() {
    let window = web_sys::window().unwrap();
    let loc = window.location();
    let href = loc.href().unwrap();
    assert!(!href.is_empty());
    assert!(href.starts_with("http"));
}

#[wasm_bindgen_test]
fn test_window_location_properties() {
    let window = web_sys::window().unwrap();
    let loc = window.location();

    let protocol = loc.protocol().unwrap();
    assert!(protocol.ends_with(':'));
    let hostname = loc.hostname().unwrap();
    assert!(!hostname.is_empty());
    let port = loc.port().unwrap();
    let _ = port;
}

#[wasm_bindgen_test]
fn test_leptos_signal_get_set() {
    use leptos::prelude::*;

    let (signal, set_signal) = create_signal(42i32);
    assert_eq!(signal.get(), 42);
    set_signal.set(100);
    assert_eq!(signal.get(), 100);
}

#[wasm_bindgen_test]
fn test_leptos_signal_string() {
    use leptos::prelude::*;

    let (signal, set_signal) = create_signal("hello".to_string());
    assert_eq!(signal.get(), "hello");
    set_signal.set("world".to_string());
    assert_eq!(signal.get(), "world");
}
