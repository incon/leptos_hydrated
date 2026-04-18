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
//! 1. Provide an initial state that is available on the very first frame by leveraging state
//!    already in the browser (e.g., from cookies or URL params).
//! 2. Simultaneously start a client-side fetch to load full data.
//! 3. Seamlessly transition from the initial state to the fetched state without UI flickering.
//!
//! ## Examples
//!
//! ### Global Hydration (Trait-based)
//!
//! ```rust
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
//!         // Read from cookie/URL synchronously
//!         ThemeState { theme: "dark".into() }
//!     }
//!     async fn fetch() -> Result<Self, ServerFnError> {
//!         // Use state already in the browser or refresh from API asynchronously
//!         Ok(ThemeState { theme: "light".into() })
//!     }
//! }
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     view! {
//!         // 1. Provide state anywhere in the tree
//!         <HydrateState<ThemeState> />
//!
//!         // 2. Consume it anywhere in the tree
//!         <MainContent />
//!     }
//! }
//!
//! #[component]
//! fn MainContent() -> impl IntoView {
//!     let state = use_hydrated::<ThemeState>();
//!     view! { <p>"Theme: " {move || state.get().theme}</p> }
//! }
//! ```
//!
//! ### Scoped Hydration
//!
//! ```rust
//! # use leptos::prelude::*;
//! # use leptos_hydrated::*;
//! # #[derive(Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
//! # struct FeatureState { data: String }
//! # impl Hydratable for FeatureState {
//! #     fn initial() -> Self { Self::default() }
//! #     async fn fetch() -> Result<Self, ServerFnError> { Ok(Self::default()) }
//! # }
//! #[component]
//! fn FeatureSection() -> impl IntoView {
//!     view! {
//!         <HydrateContext<FeatureState>>
//!             <FeatureContent />
//!         </HydrateContext<FeatureState>>
//!     }
//! }
//! # #[component] fn FeatureContent() -> impl IntoView { view! { "" } }
//! ```

use leptos::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use std::future::Future;

/// A trait for types that can be hydrated automatically.
pub trait Hydratable: Clone + Serialize + DeserializeOwned + Default + Send + Sync + 'static {
    /// The synchronous initial state (e.g., read from cookies or URL parameters).
    fn initial() -> Self;

    /// The asynchronous fetcher for refreshing or getting full data.
    fn fetch() -> impl Future<Output = Result<Self, ServerFnError>> + Send + 'static;
}

/// A wrapper for a hydrated global signal provided via context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HydratedSignal<T: 'static>(pub RwSignal<T>);

/// The core hook for creating a hydrated signal.
///
/// This creates a signal that is initialized synchronously from `ssr_value()`.
///
/// Returns a tuple of `(RwSignal<T>, LocalResource<T>)`.
pub fn use_hydrate_signal<T, Fut>(
    ssr_value: impl Fn() -> T + 'static,
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
) -> (RwSignal<T>, LocalResource<T>)
where
    T: Clone + Serialize + DeserializeOwned + Default + Send + Sync + 'static + std::fmt::Debug + PartialEq,
    Fut: Future<Output = Result<T, ServerFnError>> + Send + 'static,
{
    let initial_val = ssr_value();
    let signal = RwSignal::new(initial_val);

    // Create the resource for hydration. We use LocalResource to avoid
    // hydration mismatch warnings and redundant server-side execution.
    let resource = LocalResource::new(
        move || {
            let f = fetcher();
            async move { f.await.unwrap_or_default() }
        },
    );

    #[cfg(not(feature = "ssr"))]
    {
        // Use spawn_local to await the resource and update the signal.
        leptos::task::spawn_local(async move {
            let val = resource.await;
            signal.set(val);
        });
    }

    (signal, resource)
}

/// A version of Hydrated that uses the `Hydratable` trait for its logic.
#[component]
pub fn HydrateState<T>(
    #[prop(optional)] marker: std::marker::PhantomData<T>,
) -> impl IntoView
where
    T: Hydratable + std::fmt::Debug + PartialEq,
{
    let _ = marker;
    view! {
        <HydrateStateWith ssr_value=T::initial fetcher=T::fetch />
    }
}

/// A version of HydrateContext that uses the `Hydratable` trait for its logic.
#[component]
pub fn HydrateContext<T>(
    children: Children,
    #[prop(optional)] marker: std::marker::PhantomData<T>,
) -> impl IntoView
where
    T: Hydratable + std::fmt::Debug + PartialEq,
{
    let _ = marker;
    view! {
        <HydrateContextWith ssr_value=T::initial fetcher=T::fetch>
            {children()}
        </HydrateContextWith>
    }
}

/// A version of Hydrate that provides Global State to its descendants via context.
///
/// Use `use_hydrated::<T>()` in child components to access the state.
#[component]
pub fn HydrateStateWith<T, Fut>(
    ssr_value: impl Fn() -> T + 'static,
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
) -> impl IntoView
where
    T: Clone + Serialize + DeserializeOwned + Default + Send + Sync + 'static + std::fmt::Debug + PartialEq,
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
pub fn HydrateContextWith<T, Fut>(
    ssr_value: impl Fn() -> T + 'static,
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
    children: Children,
) -> impl IntoView
where
    T: Clone + Serialize + DeserializeOwned + Default + Send + Sync + 'static + std::fmt::Debug + PartialEq,
    Fut: Future<Output = Result<T, ServerFnError>> + Send + 'static,
{
    let (signal, resource) = use_hydrate_signal(ssr_value, fetcher);
    provide_context(HydratedSignal(signal));
    provide_context(resource);
    children()
}

/// Helper to access a signal provided by `HydrateContext` or `HydrateStateWith`.
pub fn use_hydrated<T>() -> RwSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<HydratedSignal<T>>().map(|s| s.0).expect(
        "HydratedSignal not found. Did you wrap this part of the tree in <HydrateState />, <HydrateContext />, <HydrateStateWith />, or <HydrateContextWith />?",
    )
}

/// Helper to access the resource provided by `HydrateContext` or `HydrateContextWith`.
pub fn use_hydrated_resource<T>() -> LocalResource<T>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<LocalResource<T>>().expect(
        "Hydrated Resource not found. Did you wrap this part of the tree in <HydrateContext /> or <HydrateContextWith />?",
    )
}

#[cfg(test)]
mod tests;
