#![forbid(unsafe_code)]

pub mod api;
pub mod app;
pub mod components;
pub mod pages;
pub mod state;
pub mod utils;

pub use app::*;
pub use components::*;
pub use leptos::prelude::*;

#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
