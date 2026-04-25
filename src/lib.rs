//! # Leptos Hydrated
//!
//! A library for **flicker-free interactive state hydration** in Leptos 0.8.
//!
//! `leptos_hydrated` is ideal for bootstrapping state that you **already have or can have on both sides**
//! (isomorphic data), such as cookies, URL parameters, or locally cached state. By initializing
//! signals immediately with server-provided state and synchronizing them once the browser is
//! active, you eliminate the "loading flicker" common in SSR applications.
//!
//! ## The Problem
//!
//! In SSR applications there is a gap between the server rendering HTML and the
//! client WASM initialising. If you rely on async resources to bootstrap state
//! the UI flickers from a default/loading state to the real state once JS takes
//! over.
//!
//! ## The Solution
//!
//! `leptos_hydrated` synchronises state from the server to the client by:
//!
//! 1. **Server-Side Render (SSR):** `initial()` is called on the server. The result is serialized into the HTML shell.
//! 2. **Hydration:** The client reads the serialized state from the HTML and initializes the signal immediately — **zero flicker**.
//! 3. **Synchronization:** Once the WASM is active, `initial()` is re-run on the client to synchronize with the current browser state (e.g., reading a JS-accessible cookie).
//! 4. **Lifecycle Hooks:** Use `on_hydrate` to set up browser-only event listeners (e.g., network status, window resize).
//!
//! This also handles **HTTP-only cookies**: the server reads the cookie in
//! `initial()`, injects the value, and the client never needs to touch the
//! cookie directly.
//!
//! ## Quick Start
//!
//! To use `leptos_hydrated`, you implement the [`Hydratable`] trait. This encapsulates your synchronous "seed" logic (e.g., cookies) and your asynchronous "refresh" logic (e.g., API calls).
//!
//! ```rust,no_run
//! use leptos::prelude::*;
//! # use leptos_hydrated::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
//! pub struct ThemeState {
//!     pub theme: String,
//! }
//!
//! impl Hydratable for ThemeState {
//!     fn initial() -> Self {
//!         // Use isomorphic helpers to read from cookies/query params on both sides.
//!         let theme = get_cookie("theme").unwrap_or_else(|| "dark".into());
//!         ThemeState { theme }
//!     }
//!
//!     #[cfg(not(feature = "ssr"))]
//!     fn on_hydrate(&self, state: RwSignal<Self>) {
//!         // Optional: Do something in the browser after hydration
//!     }
//! }
//!
//! #[component]
//! pub fn App() -> impl IntoView {
//!     view! {
//!         // 1. Provide state anywhere in the tree
//!         <HydrateState<ThemeState> />
//!         
//!         <MainContent />
//!     }
//! }
//!
//! #[component]
//! fn MainContent() -> impl IntoView {
//!     // 2. Consume it anywhere in the tree
//!     let state = use_hydrated::<ThemeState>();
//!     view! {
//!         <p>"Theme: " {move || state.get().theme}</p>
//!     }
//! }
//! ```
//!
//! ## Server-Side Setup
//!
//! In order for isomorphic helpers to access request data on the server, you **must** use `.leptos_routes_with_context` in your Axum server setup and call `provide_hydration_context()`.
//!
//! ```rust,ignore
//! .leptos_routes_with_context(
//!     &leptos_options,
//!     routes,
//!     || {
//!         // This initializes the hydration store from the current request
//!         leptos_hydrated::provide_hydration_context();
//!     },
//!     move || shell(),
//! )
//! ```
//!
//! ## Environment Macros
//!
//! The library provides macros to simplify environment-gated code:
//! - `isomorphic!`: Run different logic for server seed vs client hydration.
//! - `server_only!` / `client_only!`: Execute code only in one environment.
//! - `is_server()` / `is_client()`: Runtime environment checks.
//!
//! ## PWA & "Born Offline" Support
//!
//! `leptos_hydrated` supports PWAs loading from an offline shell (CSR mode) by detecting the mounting mode in your `lib.rs` and providing it via context to your components.


mod accessors;
mod components;
mod core;
mod helpers;
mod macros;
mod traits;

pub use accessors::*;
pub use components::*;
#[cfg(not(feature = "ssr"))]
pub use core::get_injected_state;
pub use core::{HydratedSignal, use_hydrate_signal};
#[allow(unused_imports)]
pub use helpers::*;
#[allow(unused_imports)]
pub use macros::*;
pub use traits::*;

/// Returns `true` if the code is currently executing on the server (SSR).
pub fn is_server() -> bool {
    #[cfg(feature = "ssr")]
    {
        true
    }
    #[cfg(not(feature = "ssr"))]
    {
        false
    }
}

/// Returns `true` if the code is currently executing in the browser (client-side).
pub fn is_client() -> bool {
    !is_server()
}

#[cfg(test)]
mod tests;
