//! # Leptos Hydrated
//!
//! A library for **flicker-free interactive state hydration** in Leptos 0.8.
//!
//! ## The Problem
//!
//! In SSR (Server-Side Rendering) applications, there is often a "gap" between the time the
//! HTML is rendered and the time the client-side JavaScript (WASM) is initialized and hydrated.
//! During this gap, if you rely on asynchronous resources to initialize your state, the UI might
//! "flicker" from a default/loading state to the actual state once the WASM takes over.
//!
//! ## The Solution
//!
//! `leptos_hydrated` provides primitives to synchronize state from the server to the client
//! synchronously during hydration. It allows you to:
//! 1. Provide an initial state that is available on the very first frame (e.g., from cookies or URL params).
//! 2. Simultaneously start a client-side fetch to load full data.
//! 3. Seamlessly transition from the initial state to the fetched state without UI flickering.
//!
//! ## Examples
//!
//! ### Using `Hydrate` (Global State)
//!
//! ```rust
//! use leptos::prelude::*;
//! use leptos_hydrated::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Default, Serialize, Deserialize)]
//! struct ThemeState { theme: String }
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     view! {
//!         // Provide global state. The ssr_value and fetcher should match
//!         // on the first render to ensure zero visual flickering.
//!         <Hydrate
//!             ssr_value=|| ThemeState { theme: "dark".into() }
//!             fetcher=|| async { Ok(ThemeState { theme: "dark".into() }) }
//!         />
//!         <MainContent />
//!     }
//! }
//!
//! #[component]
//! fn MainContent() -> impl IntoView {
//!     let state = use_hydrated::<ThemeState>();
//!     view! { <p>"Current theme: " {move || state.get().theme}</p> }
//! }
//! ```
//!
//! ### Using `HydrateContext` (Scoped State)
//!
//! ```rust
//! use leptos::prelude::*;
//! use leptos_hydrated::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Default, Serialize, Deserialize)]
//! struct UserState { name: String }
//!
//! #[component]
//! fn FeatureSection() -> impl IntoView {
//!     view! {
//!         <HydrateContext
//!             ssr_value=|| UserState { name: "Guest".into() }
//!             fetcher=|| async { Ok(UserState { name: "Guest".into() }) }
//!         >
//!             <SubComponent />
//!         </HydrateContext>
//!     }
//! }
//!
//! #[component]
//! fn SubComponent() -> impl IntoView {
//!     let user = use_hydrated::<UserState>();
//!     view! { <p>"Welcome, " {move || user.get().name}</p> }
//! }
//! ```

use leptos::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use std::future::Future;

/// A wrapper for a hydrated global signal provided via context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HydratedSignal<T: 'static>(pub RwSignal<T>);

/// The core hook for creating a hydrated signal.
///
/// This creates a signal that is initialized synchronously from `ssr_value` on both
/// server and client, and then updated with the result of `fetcher` once it resolves
/// on the client.
///
/// This is the foundation for flicker-free hydration.
///
/// Returns a tuple of `(RwSignal<T>, LocalResource<T>)`.
pub fn use_hydrate_signal<T, Fut>(
    ssr_value: impl Fn() -> T + 'static,
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
) -> (RwSignal<T>, LocalResource<T>)
where
    T: Clone + Serialize + DeserializeOwned + Default + Send + Sync + 'static,
    Fut: Future<Output = Result<T, ServerFnError>> + Send + 'static,
{
    // Create the resource for hydration. We use LocalResource to avoid
    // hydration mismatch warnings and redundant server-side execution,
    // as the initial state is already provided by ssr_value().
    let resource = LocalResource::new(
        move || {
            let f = fetcher();
            async move { f.await.unwrap_or_default() }
        },
    );

    // Evaluate ssr_value() to get the initial synchronous state.
    // On the server, this builds the first frame. On the client, this builds
    // the hydration frame (matching the server) without prematurely reading the resource.
    let initial_val = ssr_value();

    let signal = RwSignal::new(initial_val);


    #[cfg(not(feature = "ssr"))]
    {
        // Use spawn_local to await the resource and update the signal.
        // This ensures the update happens as soon as data is available,
        // and is more reliably testable than a reactive Effect.
        leptos::task::spawn_local(async move {
            let val = resource.await;
            signal.set(val);
        });
    }

    (signal, resource)
}

/// A version of Hydrated that provides Global State to its descendants via context.
///
/// Use `use_hydrated::<T>()` in child components to access the state.
#[component]
pub fn Hydrate<T, Fut>(
    ssr_value: impl Fn() -> T + 'static,
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
) -> impl IntoView
where
    T: Clone + Serialize + DeserializeOwned + Default + Send + Sync + 'static,
    Fut: Future<Output = Result<T, ServerFnError>> + Send + 'static,
{
    let (signal, _) = use_hydrate_signal(ssr_value, fetcher);
    provide_context(HydratedSignal(signal));
}

/// A version of Hydrated that provides the signal via Context to all descendants.
///
/// Use `use_hydrated::<T>()` or `use_hydrated_resource::<T>()` in child components
/// to access the state and resource.
#[component]
pub fn HydrateContext<T, Fut>(
    ssr_value: impl Fn() -> T + 'static,
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
    children: Children,
) -> impl IntoView
where
    T: Clone + Serialize + DeserializeOwned + Default + Send + Sync + 'static,
    Fut: Future<Output = Result<T, ServerFnError>> + Send + 'static,
{
    let (signal, resource) = use_hydrate_signal(ssr_value, fetcher);
    provide_context(HydratedSignal(signal));
    provide_context(resource);
    children()
}

/// Helper to access a signal provided by `HydrateContext` or `Hydrate`.
pub fn use_hydrated<T>() -> RwSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<HydratedSignal<T>>().map(|s| s.0).expect(
        "HydratedSignal not found. Did you wrap this part of the tree in <HydrateContext /> or <Hydrate />?",
    )
}

/// Helper to access the resource provided by `HydrateContext`.
pub fn use_hydrated_resource<T>() -> LocalResource<T>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<LocalResource<T>>().expect(
        "Hydrated Resource not found. Did you wrap this part of the tree in <HydrateContext />?",
    )
}

#[cfg(test)]
mod tests;
