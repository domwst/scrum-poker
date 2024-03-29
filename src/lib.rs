pub mod app;
pub mod error_template;
pub mod macros;
if_backend! {
    pub mod fileserv;
    pub mod random_nickname;
}
pub mod components;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
