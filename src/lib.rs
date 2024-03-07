#[cfg(feature = "ssr")]
pub mod auth_model;
#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod fileserv;
#[cfg(feature = "ssr")]
pub mod state;

// #[cfg(feature = "ssr")]
// pub mod ws;
// #[cfg(feature = "ssr")]
// pub mod messaging;

pub mod app;
pub mod error_template;
pub mod message_model;
pub mod rooms_manager;
pub mod user_model;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
