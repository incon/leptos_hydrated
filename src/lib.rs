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
//! 5. **WASM Bundle Optimization:** Use `#[hydrated_server]` to eliminate `serde_json` from your client-side WASM bundle.
//!
//! ## Hydration Scopes
//!
//! `leptos_hydrated` offers three levels of state scope, ordered by increasing granularity:
//!
//! ### 1. Local
//!
//! Each call to `hydrated_signal` creates a new, independent hydrated signal.
//! This is the primary entry point for hydrated state. Synchronization is
//! handled automatically via a deterministic hydration counter.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_hydrated::*;
//! # #[derive(Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)] struct MyState;
//! # impl Hydratable for MyState { fn initial() -> Self { Self } }
//! #[component]
//! fn MyComponent() -> impl IntoView {
//!     // Always creates a new, independent signal
//!     let state = hydrated_signal(MyState::initial());
//!     // ...
//! }
//! ```
//!
//! ### 2. Scoped
//!
//! Wrap a section of your component tree with `<HydratedContext<T>>` to share
//! a hydrated signal. Use `use_hydrated_context<T>()` in descendants to access it.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_hydrated::*;
//! # #[derive(Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)] struct MyState;
//! # impl Hydratable for MyState { fn initial() -> Self { Self } }
//! #[component]
//! fn Feature() -> impl IntoView {
//!     view! {
//!         <HydratedContext<MyState>>
//!             <Descendant />
//!         </HydratedContext<MyState>>
//!     }
//! }
//!
//! #[component]
//! fn Descendant() -> impl IntoView {
//!     // Access the shared signal from context (returns Option<RwSignal<T>>)
//!     let state = use_hydrated_context::<MyState>();
//!     // ...
//! }
//! ```
//!
//! ### 3. Global
//!
//! Use `<HydratedContext<T> global=true />` (typically in your app shell). This provides the state globally across your entire application.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_hydrated::*;
//! # #[derive(Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)] struct MyState;
//! # impl Hydratable for MyState { fn initial() -> Self { Self } }
//! #[component]
//! fn App() -> impl IntoView {
//!     view! {
//!         <HydratedContext<MyState> global=true />
//!         // ...
//!     }
//! }
//! ```
//!
//! ## Bundle Size Optimization
//!
//! One of the biggest contributors to Leptos WASM bundle size is `serde_json`. `leptos_hydrated` provides a custom codec and macro to eliminate this weight from your client-side binary.
//!
//! ### `#[hydrated_server]`
//!
//! Replace standard `#[server]` with `#[hydrated_server]` to opt into the `BrowserJson` protocol. This uses the browser's native `JSON.parse` and `JSON.stringify` on the client, and `serde_json` on the server.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_hydrated::*;
//! # #[derive(Clone, serde::Serialize, serde::Deserialize)] struct MyData;
//! #[hydrated_server]
//! pub async fn my_server_fn(data: MyData) -> Result<MyData, ServerFnError> {
//!     Ok(data)
//! }
//! ```
//!
//! By using this macro, you can often save **150 KB - 200 KB** on your final WASM binary. It is fully compatible with `ActionForm` because it uses standard URL encoding for inputs and optimized JSON for outputs.
//!
//!
//! ## Quick Start
//!
//! Implement the [`Hydratable`] trait to define how your state is initialized and synchronized.
//!
//! ```rust,no_run
//! use leptos::prelude::*;
//! use leptos_hydrated::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
//! pub struct ThemeState {
//!     pub theme: String,
//! }
//!
//! impl Hydratable for ThemeState {
//!     fn initial() -> Self {
//!         // Use isomorphic helpers to read from cookies on both sides.
//!         let theme = get_cookie("theme").unwrap_or_else(|| "dark".into());
//!         ThemeState { theme }
//!     }
//!
//!     #[cfg(not(feature = "ssr"))]
//!     fn on_hydrate(&self) {
//!         // Optional: Execute code in the browser after hydration
//!         leptos::logging::log!("Hydrated theme: {}", self.theme);
//!     }
//! }
//!
//! #[component]
//! pub fn App() -> impl IntoView {
//!     view! {
//!         // Provide state globally
//!         <HydratedContext<ThemeState> global=true />
//!
//!         <MainContent />
//!     }
//! }
//!
//! #[component]
//! fn MainContent() -> impl IntoView {
//!     // Consume it anywhere in the tree
//!     let state = hydrated_signal(ThemeState::initial());
//!     view! {
//!         <p>"Theme: " {move || state.get().theme}</p>
//!     }
//! }
//! ```
//!
//! ## Server-Side Setup
//!
//! You **must** add the `.hydrated()` middleware to your Axum router to enable state injection.
//!
//! ```rust,ignore
//! # #[cfg(feature = "ssr")]
//! # {
//! # use axum::Router;
//! # use leptos_hydrated::HydratedRouterExt;
//! # use leptos::prelude::LeptosOptions;
//! # let leptos_options = LeptosOptions::builder().output_name("app").build();
//! let app = Router::new()
//!     .leptos_routes(...)
//!     .fallback(...)
//!     .hydrated() // <--- Add this before .with_state()
//!     .with_state(leptos_options);
//! # }
//! ```
//!
//!
//! ## PWA & "Born Offline" Support
//!
//! `leptos_hydrated` supports PWAs loading from an offline shell (CSR mode) by detecting the mounting mode in your `lib.rs` and providing it via context to your components.

pub mod components;
pub mod codec;
mod core;
mod helpers;
mod macros;
#[cfg(feature = "ssr")]
mod ssr;
mod traits;

pub use components::HydratedContext;
pub use core::{hydrated_signal, use_hydrated_context};

#[allow(unused_imports)]
pub use helpers::*;
#[allow(unused_imports)]
pub use macros::*;
#[cfg(feature = "ssr")]
pub use ssr::*;
pub use traits::*;
pub use leptos_hydrated_macro::{hydrate, ssr, hydrated_server};

#[cfg(test)]
mod tests;
