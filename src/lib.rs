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
//! ## Hydration Scopes
//!
//! `leptos_hydrated` offers three levels of state scope, ordered by increasing granularity:
//!
//! ### 1. Local
//!
//! Use `hydrated_signal` directly in a component. This creates a hydrated signal that is unique to this component instance and is **not** shared via context.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_hydrated::*;
//! # #[derive(Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)] struct MyState;
//! # impl Hydratable for MyState { fn initial() -> Self { Self } }
//! #[component]
//! fn MyComponent() -> impl IntoView {
//!     let state = hydrated_signal(MyState::initial());
//!     // ...
//! }
//! ```
//!
//! ### 2. Scoped
//!
//! Wrap a section of your component tree with `<HydratedContext<T>>`. This provides the hydrated state to all descendants in that subtree.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_hydrated::*;
//! # #[derive(Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)] struct MyState;
//! # impl Hydratable for MyState { fn initial() -> Self { Self } }
//! # #[component] fn Descendant() -> impl IntoView { view! { "Descendant" } }
//! #[component]
//! fn Feature() -> impl IntoView {
//!     view! {
//!         <HydratedContext<MyState>>
//!             <Descendant />
//!         </HydratedContext<MyState>>
//!     }
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
//!     fn on_hydrate(&self, state: RwSignal<Self>) {
//!         // Optional: Do something in the browser after hydration
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
//! ## Environment Utilities
//!
//! - `isomorphic!`: Run different logic for server seed vs client hydration.
//! - `use_hydrated_context<T>()`: Accesses the hydrated state from context.
//! - `inject_state(&value)`: Manually inject a state from the server (SSR only).
//! - `use_injected_state<T>()`: Reads the next available injected state from the server (client-side only).
//!
//! ### Example: Manual Injection with `isomorphic!`
//!
//! ```rust,no_run
//! # use leptos_hydrated::*;
//! # #[derive(serde::Serialize, serde::Deserialize)] struct MyState { count: i32 }
//! let my_value = isomorphic! {
//!     state => {
//!         let value = MyState { count: 42 };
//!         inject_state(&value);
//!         value
//!     },
//!     hydrate => {
//!         use_injected_state::<MyState>().unwrap_or_else(|| MyState { count: 0 })
//!     }
//! };
//! ```
//!
//!
//! ## PWA & "Born Offline" Support
//!
//! `leptos_hydrated` supports PWAs loading from an offline shell (CSR mode) by detecting the mounting mode in your `lib.rs` and providing it via context to your components.

mod accessors;
mod components;
mod core;
mod helpers;
mod macros;
#[cfg(feature = "ssr")]
mod ssr;
mod traits;

pub use accessors::Hydrated;
pub use components::HydratedContext;
pub use core::{hydrated_signal, use_hydrated_context, HydrateSignal, use_injected_state, inject_state};
#[allow(unused_imports)]
pub use helpers::*;
#[allow(unused_imports)]
pub use macros::*;
#[cfg(feature = "ssr")]
pub use ssr::*;
pub use traits::*;

#[cfg(test)]
mod tests;
