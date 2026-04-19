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
//! 3. Optionally running `fetch()` after hydration to refresh with the latest
//!    client-side state (e.g. re-reading a JS-accessible cookie). When `fetch`
//!    is not needed, the default returns `None` and the injected value is kept.
//!
//! This also handles **HTTP-only cookies**: the server reads the cookie in
//! `initial()`, injects the value, and the client never needs to touch the
//! cookie directly.
//!
//! ## Two Modes
//!
//! | Mode | `fetch()` | Use when |
//! |------|-----------|----------|
//! | Injection-only | `None` (default) | Server value is the source of truth (HTTP-only cookies, session tokens) |
//! | Injection + refresh | `Some(Ok(v))` | Client can also re-read the same state (JS-readable cookies, URL params) |
//!
//! ## Examples
//!
//! ### Injection-only (HTTP-only cookie / session)
//!
//! ```rust
//! use leptos::prelude::*;
//! use leptos_hydrated::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
//! pub struct SessionState {
//!     pub user_id: Option<u64>,
//! }
//!
//! impl Hydratable for SessionState {
//!     fn initial() -> Self {
//!         // On SSR: read from HTTP-only session cookie in request headers.
//!         // The value is injected into the HTML — the client never reads the cookie.
//!         SessionState { user_id: None }
//!     }
//!     // fetch() defaults to None — injected server value is kept, no refresh.
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
//!
//! ### Injection + client refresh (JS-readable cookie / URL param)
//!
//! ```rust
//! # use leptos::prelude::*;
//! # use leptos_hydrated::*;
//! # use serde::{Serialize, Deserialize};
//! #[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
//! pub struct ThemeState { pub theme: String }
//!
//! impl Hydratable for ThemeState {
//!     fn initial() -> Self {
//!         // On SSR: read from request headers.
//!         // On client: read from browser APIs (document.cookie, window.location).
//!         ThemeState { theme: "light".into() }
//!     }
//!     fn fetch() -> impl std::future::Future<Output = Option<Result<Self, ServerFnError>>> + Send + 'static {
//!         // Re-read from the same client-side source after hydration.
//!         async { Some(Ok(ThemeState { theme: "light".into() })) }
//!     }
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

// ---------------------------------------------------------------------------
// Helpers: type-stable DOM id, serialization, and injection reading
// ---------------------------------------------------------------------------

fn type_hydration_id<T: 'static>() -> String {
    std::any::type_name::<T>()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(feature = "ssr")]
fn serialize_for_injection<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

#[cfg(all(not(feature = "ssr"), feature = "hydrate"))]
fn read_injected_state<T: DeserializeOwned>(id: &str) -> Option<T> {
    use js_sys::JSON;
    use wasm_bindgen::JsCast as _;
    use wasm_bindgen::JsValue;

    let doc = document();
    let script_id = format!("__lh_{}", id);

    let el: JsValue = js_sys::Reflect::get(&doc, &JsValue::from_str("getElementById"))
        .ok()
        .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
        .and_then(|f| f.call1(&doc, &JsValue::from_str(&script_id)).ok())
        .filter(|v: &JsValue| !v.is_null() && !v.is_undefined())?;

    let text = js_sys::Reflect::get(&el, &JsValue::from_str("textContent"))
        .ok()
        .and_then(|v| v.as_string())?;

    let js_val = JSON::parse(&text).ok()?;
    serde_wasm_bindgen::from_value(js_val).ok()
}

#[cfg(all(not(feature = "ssr"), not(feature = "hydrate")))]
fn read_injected_state<T: DeserializeOwned>(_id: &str) -> Option<T> {
    None
}

// ---------------------------------------------------------------------------
// Hydratable trait
// ---------------------------------------------------------------------------

/// A trait for types that can be hydrated automatically.
pub trait Hydratable: Clone + Serialize + DeserializeOwned + Default + Send + Sync + 'static {
    /// Read from request details (cookies, URL params).
    ///
    /// - On SSR: read from HTTP request headers/URI. The result is serialised
    ///   into the HTML so the client never needs to re-compute it.
    /// - On client: used as a fallback when no injected value is found (CSR-only).
    fn initial() -> Self;

    /// Optional async client-side refresh after hydration.
    ///
    /// - `None` (default): keep the injected server value. No network call.
    ///   Ideal for HTTP-only cookies and session tokens.
    /// - `Some(Ok(v))`: update the signal with `v` after hydration.
    ///   Use when the client can re-read the same state (JS cookies, URL params).
    /// - `Some(Err(_))`: log/ignore; signal retains its current value.
    fn fetch() -> impl Future<Output = Option<Result<Self, ServerFnError>>> + Send + 'static {
        async { None }
    }
}

// ---------------------------------------------------------------------------
// HydratedSignal wrapper
// ---------------------------------------------------------------------------

/// A wrapper for a hydrated global signal provided via context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HydratedSignal<T: 'static>(pub RwSignal<T>);

// ---------------------------------------------------------------------------
// Core hook
// ---------------------------------------------------------------------------

/// The core hook for creating a hydrated signal.
///
/// `ssr_value` is the synchronous initial value. `fetcher` returns
/// `Option<Result<T, E>>`:
/// - `None`        → no update after initial hydration
/// - `Some(Ok(v))` → signal is updated to `v`
/// - `Some(Err(_))` → signal retains its current value
///
/// Returns `(RwSignal<T>, LocalResource<Option<T>>)`.
pub fn use_hydrate_signal<T, Fut>(
    ssr_value: impl Fn() -> T + 'static,
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
) -> (RwSignal<T>, LocalResource<Option<T>>)
where
    T: Clone + Serialize + DeserializeOwned + Default + Send + Sync + PartialEq + 'static,
    Fut: Future<Output = Option<Result<T, ServerFnError>>> + Send + 'static,
{
    let initial_val = ssr_value();
    let signal = RwSignal::new(initial_val);

    let resource = LocalResource::new(move || {
        let f = fetcher();
        async move {
            match f.await {
                Some(Ok(v)) => Some(v),
                _ => None,
            }
        }
    });

    #[cfg(not(feature = "ssr"))]
    {
        leptos::task::spawn_local(async move {
            if let Some(val) = resource.await {
                signal.set(val);
            }
        });
    }

    (signal, resource)
}

/// Reads the raw JSON string from the injected script tag on the client.
/// Used to prevent hydration mismatches by ensuring the client-side view matches the SSR view.
#[cfg(all(not(feature = "ssr"), feature = "hydrate"))]
fn read_raw_injected_state(id: &str) -> Option<String> {
    let script_id = format!("__lh_{}", id);
    document()
        .get_element_by_id(&script_id)
        .and_then(|el| el.text_content())
}

#[cfg(all(not(feature = "ssr"), not(feature = "hydrate")))]
fn read_raw_injected_state(_id: &str) -> Option<String> {
    None
}

// ---------------------------------------------------------------------------
// Trait-based components (always inject)
// ---------------------------------------------------------------------------

/// Provides global hydrated state using the [`Hydratable`] trait.
///
/// Injects the server value into an inline `<script>` tag so the client can
/// read it immediately — no flicker. The script tag is always rendered on
/// both SSR and client to keep the DOM structure identical for hydration.
/// Use `use_hydrated::<T>()` in any descendant to access the signal.
#[component]
pub fn HydrateState<T>(
    #[prop(optional)] marker: std::marker::PhantomData<T>,
) -> impl IntoView
where
    T: Hydratable + PartialEq,
{
    let _ = marker;
    let id = type_hydration_id::<T>();
    let script_id = format!("__lh_{}", id);

    #[cfg(feature = "ssr")]
    let initial_val = T::initial();
    #[cfg(not(feature = "ssr"))]
    let initial_val = read_injected_state::<T>(&id).unwrap_or_else(T::initial);

    #[cfg(feature = "ssr")]
    let json = serialize_for_injection(&initial_val);
    #[cfg(not(feature = "ssr"))]
    let json = read_raw_injected_state(&id).unwrap_or_default();

    let cloned = initial_val.clone();
    let (signal, resource) = use_hydrate_signal(move || cloned.clone(), || T::fetch());
    provide_context(HydratedSignal(signal));
    provide_context(resource);

    // Script tag rendered on BOTH sides — same DOM node count for hydration.
    // Content is populated on SSR; empty on client (already read above).
    view! {
        <script type="application/json" id={script_id} inner_html={json} />
    }
}

/// Provides scoped hydrated state using the [`Hydratable`] trait.
///
/// Injects the server value and renders children inside the same component.
/// Use `use_hydrated::<T>()` in any child to access the signal.
#[component]
pub fn HydrateContext<T>(
    children: Children,
    #[prop(optional)] marker: std::marker::PhantomData<T>,
) -> impl IntoView
where
    T: Hydratable + PartialEq,
{
    let _ = marker;
    let id = type_hydration_id::<T>();
    let script_id = format!("__lh_{}", id);

    #[cfg(feature = "ssr")]
    let initial_val = T::initial();
    #[cfg(not(feature = "ssr"))]
    let initial_val = read_injected_state::<T>(&id).unwrap_or_else(T::initial);

    #[cfg(feature = "ssr")]
    let json = serialize_for_injection(&initial_val);
    #[cfg(not(feature = "ssr"))]
    let json = read_raw_injected_state(&id).unwrap_or_default();

    let cloned = initial_val.clone();
    let (signal, resource) = use_hydrate_signal(move || cloned.clone(), || T::fetch());
    provide_context(HydratedSignal(signal));
    provide_context(resource);

    view! {
        {children()}
        <script type="application/json" id={script_id} inner_html={json} />
    }
}

// ---------------------------------------------------------------------------
// Manual "With" components
// ---------------------------------------------------------------------------

/// Manual global state provider (closure-based).
///
/// Optionally provide `server_value` to inject an SSR-only value into the HTML
/// (e.g. from an HTTP-only cookie). When `server_value` is `None`, `ssr_value`
/// is used on both sides. The `<script>` tag is always rendered to keep the
/// DOM structure identical on both sides.
#[component]
pub fn HydrateStateWith<T, Fut>(
    /// Client-side initial value. Also used on SSR when `server_value` is `None`.
    ssr_value: impl Fn() -> T + 'static,
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
    /// SSR-only override. When provided the value is injected and the client
    /// reads from the injection instead of calling `ssr_value`.
    #[prop(optional)] server_value: Option<T>,
) -> impl IntoView
where
    T: Clone + Serialize + DeserializeOwned + Default + Send + Sync + PartialEq + 'static,
    Fut: Future<Output = Option<Result<T, ServerFnError>>> + Send + 'static,
{
    let id = type_hydration_id::<T>();
    let script_id = format!("__lh_{}", id);

    #[cfg(feature = "ssr")]
    let (initial_val, json) = {
        let val = server_value.unwrap_or_else(&ssr_value);
        let json = serialize_for_injection(&val);
        (val, json)
    };

    #[cfg(not(feature = "ssr"))]
    let (initial_val, json) = {
        let _ = server_value;
        let val = read_injected_state::<T>(&id).unwrap_or_else(ssr_value);
        let json = read_raw_injected_state(&id).unwrap_or_default();
        (val, json)
    };

    let cloned = initial_val.clone();
    let (signal, resource) = use_hydrate_signal(move || cloned.clone(), fetcher);
    provide_context(HydratedSignal(signal));
    provide_context(resource);

    view! {
        <script type="application/json" id={script_id} inner_html={json} />
    }
}

/// Manual scoped state provider (closure-based).
///
/// Optionally provide `server_value` to inject an SSR-only value into the HTML.
/// The `<script>` tag is always rendered to keep DOM structure consistent.
#[component]
pub fn HydrateContextWith<T, Fut>(
    ssr_value: impl Fn() -> T + 'static,
    fetcher: impl Fn() -> Fut + Send + Sync + 'static,
    children: Children,
    #[prop(optional)] server_value: Option<T>,
) -> impl IntoView
where
    T: Clone + Serialize + DeserializeOwned + Default + Send + Sync + PartialEq + 'static,
    Fut: Future<Output = Option<Result<T, ServerFnError>>> + Send + 'static,
{
    let id = type_hydration_id::<T>();
    let script_id = format!("__lh_{}", id);

    #[cfg(feature = "ssr")]
    let (initial_val, json) = {
        let val = server_value.unwrap_or_else(&ssr_value);
        let json = serialize_for_injection(&val);
        (val, json)
    };

    #[cfg(not(feature = "ssr"))]
    let (initial_val, json) = {
        let _ = server_value;
        let val = read_injected_state::<T>(&id).unwrap_or_else(ssr_value);
        let json = read_raw_injected_state(&id).unwrap_or_default();
        (val, json)
    };

    let cloned = initial_val.clone();
    let (signal, resource) = use_hydrate_signal(move || cloned.clone(), fetcher);
    provide_context(HydratedSignal(signal));
    provide_context(resource);

    view! {
        {children()}
        <script type="application/json" id={script_id} inner_html={json} />
    }
}

// ---------------------------------------------------------------------------
// Context accessors
// ---------------------------------------------------------------------------

/// Access a signal provided by any `Hydrate*` component.
///
/// # Panics
/// Panics if no `HydratedSignal<T>` is found in context.
/// Use [`try_use_hydrated`] for a non-panicking alternative.
pub fn use_hydrated<T>() -> RwSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<HydratedSignal<T>>().map(|s| s.0).expect(
        &format!(
            "HydratedSignal<{}> not found. Did you wrap this part of the tree in <HydrateState<{0}> />, <HydrateContext<{0}> />, <HydrateStateWith<{0}> />, or <HydrateContextWith<{0}> />?",
            std::any::type_name::<T>()
        )
    )
}

/// Non-panicking variant of [`use_hydrated`]. Returns `None` if no context is found.
pub fn try_use_hydrated<T>() -> Option<RwSignal<T>>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<HydratedSignal<T>>().map(|s| s.0)
}

/// Access the resource provided by any `Hydrate*` component.
///
/// # Panics
/// Panics if no resource is found in context.
/// Use [`try_use_hydrated_resource`] for a non-panicking alternative.
pub fn use_hydrated_resource<T>() -> LocalResource<Option<T>>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<LocalResource<Option<T>>>().unwrap_or_else(|| {
        panic!(
            "Hydrated Resource<{}> not found. Did you wrap this part of the tree in <HydrateState<{0}> />, <HydrateContext<{0}> />, <HydrateStateWith<{0}> />, or <HydrateContextWith<{0}> />?",
            std::any::type_name::<T>()
        )
    })
}

/// Non-panicking variant of [`use_hydrated_resource`]. Returns `None` if no context is found.
pub fn try_use_hydrated_resource<T>() -> Option<LocalResource<Option<T>>>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<LocalResource<Option<T>>>()
}

#[cfg(test)]
mod tests;
