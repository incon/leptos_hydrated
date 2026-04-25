//! # Leptos Hydrated
//!
//! A library for **flicker-free interactive state hydration** in Leptos 0.8.
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
//! 1. Calling `initial()` on the server and serialising the value into a
//!    `<script>` tag embedded in the HTML.
//! 2. On the client, deserialising that value as the signal's first frame —
//!    no async wait, no flicker.
//! 3. After hydration, the client re-runs `initial()` to synchronize with the
//!    current client-side state (e.g. re-reading a JS-accessible cookie).
//!
//! This also handles **HTTP-only cookies**: the server reads the cookie in
//! `initial()`, injects the value, and the client never needs to touch the
//! cookie directly.
//!
//! ## Example
//!
//! ```rust,no_run
//! use leptos::prelude::*;
//! # use leptos_hydrated::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
//! pub struct SessionState {
//!     pub user_id: Option<u64>,
//! }
//!
//! impl Hydratable for SessionState {
//!     fn initial() -> Self {
//!         // Uses the isomorphic helper to read a cookie on both server and client.
//!         let user_id = get_cookie("user_id").and_then(|id| id.parse().ok());
//!         SessionState { user_id }
//!     }
//! }
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     view! {
//!         <HydrateState<SessionState> />
//!         <Profile />
//!     }
//! }
//! # #[component] fn Profile() -> impl IntoView { view! { "" } }
//! ```

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
