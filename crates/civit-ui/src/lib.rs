#![forbid(unsafe_code)]

pub mod api;
pub mod app;
pub mod components;
pub mod error_capture;
pub mod pages;
pub mod state;
pub mod utils;

pub use app::*;
pub use components::*;
pub use leptos::prelude::*;

#[cfg(all(test, target_arch = "wasm32", feature = "csr"))]
mod wasm_tests;

#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    error_capture::install_global_error_listeners();
    let _ = js_sys::eval(
        "if ('serviceWorker' in navigator) { navigator.serviceWorker.register('/sw.js'); }",
    );
    leptos::mount::mount_to_body(App);
}
