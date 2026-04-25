pub mod app;
pub mod db;
pub mod states;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let body = document.body().unwrap();
    let has_ui = body.query_selector(":not(script)").unwrap().is_some();

    if !has_ui {
        leptos::logging::log!("Mounting app (CSR)...");
        leptos::mount::mount_to_body(move || {
            leptos::view! { <Pwa was_hydrated=false><App /></Pwa> }
        });
    } else {
        leptos::logging::log!("Hydrating app...");
        leptos::mount::hydrate_body(move || {
            leptos::view! { <Pwa was_hydrated=true><App /></Pwa> }
        });
    }
}
