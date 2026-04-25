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
//!
//!
//! ### Scoped Hydration
//!
//! ```rust,no_run
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
//!
//! ## Server-Side Setup
//!
//! For isomorphic helpers like [`get_cookie`] and [`get_query_param`] to work on the server,
//! you **must** use `.leptos_routes_with_context` in your Axum server setup.
//! This provides the necessary request and response context to the library.
//!
//! ```rust,no_run
//! # #[cfg(feature = "ssr")]
//! # async fn setup() {
//! # use axum::Router;
//! # use leptos::prelude::*;
//! # use leptos_axum::*;
//! # fn dummy<T>() -> T { panic!() }
//! # let leptos_options: LeptosOptions = dummy();
//! # let routes = dummy();
//! # let shell = || "".to_string();
//! let app: Router = Router::new()
//!     .leptos_routes_with_context(
//!         &leptos_options,
//!         routes,
//!         || {}, // Additional context providers
//!         move || shell(),
//!     )
//!     .with_state(leptos_options);
//! # }
//! ```
//!

use leptos::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

#[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
mod mock_state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    thread_local! {
        pub static COOKIES: RefCell<HashMap<String, String>> = Default::default();
        pub static QUERY_PARAMS: RefCell<HashMap<String, String>> = Default::default();
    }
}

// ---------------------------------------------------------------------------
// Isomorphic Helpers
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! hydrated {
    (server => $server:expr, client => $client:expr $(,)?) => {{
        #[cfg(feature = "ssr")]
        {
            $server
        }
        #[cfg(not(feature = "ssr"))]
        {
            $client
        }
    }};
}

/// Executes the given block only on the server.
/// Returns the result of the block on the server, or `()` in the browser.
/// This is useful for side-effects where you don't need an `Option`.
#[macro_export]
macro_rules! server_only {
    ($($t:tt)*) => {
        {
            #[cfg(feature = "ssr")]
            { $($t)*; }
            ()
        }
    }
}

/// Executes the given block only in the browser.
/// Returns the result of the block in the browser, or `()` on the server.
/// This is useful for side-effects where you don't need an `Option`.
#[macro_export]
macro_rules! client_only {
    ($($t:tt)*) => {
        {
            #[cfg(not(feature = "ssr"))]
            {
                $($t)*
            }
            #[cfg(feature = "ssr")]
            {
                ()
            }
        }
    };
}

/// Returns `true` if running on the server.
pub fn is_server() -> bool {
    cfg!(feature = "ssr")
}

/// Returns `true` if running in the browser.
pub fn is_client() -> bool {
    !cfg!(feature = "ssr")
}

// ---------------------------------------------------------------------------
// Helpers: type-stable DOM id, serialization, and injection reading
// ---------------------------------------------------------------------------

pub(crate) fn type_hydration_id<T: 'static>() -> String {
    std::any::type_name::<T>()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
}

#[cfg(feature = "ssr")]
pub(crate) fn serialize_for_injection<T: Serialize>(value: &T) -> String {
    leptos::serde_json::to_string(value).unwrap_or_default()
}

#[cfg(not(feature = "ssr"))]
fn read_injected_state<T: DeserializeOwned>(id: &str) -> Option<T> {
    #[cfg(all(target_arch = "wasm32", feature = "hydrate"))]
    {
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

    #[cfg(any(not(target_arch = "wasm32"), not(feature = "hydrate")))]
    {
        let _ = id;
        None
    }
}

// ---------------------------------------------------------------------------
// Hydratable trait
// ---------------------------------------------------------------------------

/// A trait for types that can be hydrated automatically.
pub trait Hydratable:
    Clone + Serialize + DeserializeOwned + Default + Send + Sync + 'static
{
    /// Read from request details using isomorphic helpers like [`get_cookie`] or [`get_query_param`].
    ///
    /// - On SSR: read from HTTP request headers/URI. The result is serialised
    ///   into the HTML so the client never needs to re-compute it.
    /// - On client: used as a fallback when no injected value is found (CSR-only),
    ///   and re-run after hydration to synchronise with the client-side state.
    fn initial() -> Self;

    /// Optional hook called on the client after the signal is created and hydrated.
    #[cfg(any(feature = "hydrate", feature = "csr"))]
    fn on_hydrate(&self, _signal: RwSignal<Self>) {}
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
/// This hook automatically manages signal hydration from a `LocalResource`
/// that calls `T::initial()`.
///
/// Returns `(RwSignal<T>, LocalResource<Option<T>>)`
pub fn use_hydrate_signal<T>() -> (RwSignal<T>, LocalResource<Option<T>>)
where
    T: Hydratable + PartialEq,
{
    #[cfg(not(feature = "ssr"))]
    let (initial_val, _is_injected) = {
        let id = type_hydration_id::<T>();
        let injected = read_injected_state::<T>(&id);
        let is_inj = injected.is_some();
        let val = injected.unwrap_or_else(T::initial);
        (val, is_inj)
    };

    #[cfg(feature = "ssr")]
    let (initial_val, _is_injected) = (T::initial(), false);

    let signal = RwSignal::new(initial_val.clone());
    let first_run = StoredValue::new(true);

    let resource = LocalResource::new(move || {
        let current_val = signal.get();
        let is_first = first_run.get_value();

        async move {
            if is_first {
                first_run.set_value(false);
                Some(T::initial())
            } else {
                Some(current_val)
            }
        }
    });

    #[cfg(all(not(feature = "ssr"), any(not(feature = "ssr"), not(test))))]
    {
        let resource_cloned = resource.clone();
        leptos::task::spawn_local(async move {
            if let Some(val) = resource_cloned.await {
                signal.set(val);
            }
        });
    }

    #[cfg(not(feature = "ssr"))]
    {
        initial_val.on_hydrate(signal);
    }

    (signal, resource)
}

// ---------------------------------------------------------------------------
// Isomorphic Helpers
// ---------------------------------------------------------------------------

/// Reads a cookie by name on both server and client.
///
/// - **SSR:** Reads from `http::request::Parts` (requires server setup with `leptos_routes_with_context`).
/// - **Client:** Reads from `document.cookie`.
pub fn get_cookie(name: &str) -> Option<String> {
    #[cfg(all(target_arch = "wasm32", not(feature = "ssr")))]
    {
        let cookies = js_sys::Reflect::get(&document(), &wasm_bindgen::JsValue::from_str("cookie"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        return parse_key_value_pair(&cookies, name, "; ");
    }

    #[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
    {
        #[cfg(feature = "ssr")]
        {
            use http::header::COOKIE;
            use http::request::Parts;
            use leptos::prelude::use_context;

            if let Some(val) = use_context::<Parts>().and_then(|parts| {
                parts
                    .headers
                    .get(COOKIE)
                    .and_then(|h| h.to_str().ok())
                    .and_then(|cookies| parse_key_value_pair(cookies, name, "; "))
            }) {
                return Some(val);
            }
        }
        mock_state::COOKIES.with(|c| c.borrow().get(name).cloned())
    }
}

/// Reads a URL query parameter by name on both server and client.
///
/// - **SSR:** Tries reading from the current request URI first. If not found, falls back
///   to the `Referer` header. This is useful for server functions where the query parameters
///   from the page that made the request are needed.
/// - **Client:** Reads from `window.location.search`.
pub fn get_query_param(name: &str) -> Option<String> {
    #[cfg(all(target_arch = "wasm32", not(feature = "ssr")))]
    {
        let search = window().location().search().ok().unwrap_or_default();
        return web_sys::UrlSearchParams::new_with_str(&search)
            .ok()
            .and_then(|p| p.get(name));
    }

    #[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
    {
        #[cfg(feature = "ssr")]
        {
            use http::header::REFERER;
            use http::request::Parts;
            use leptos::prelude::use_context;

            if let Some(parts) = use_context::<Parts>() {
                // 1. Try current URI query
                if let Some(q) = parts.uri.query() {
                    if let Some(val) = parse_key_value_pair(q, name, "&") {
                        return Some(val);
                    }
                }

                // 2. Try Referer header query
                if let Some(val) = parts
                    .headers
                    .get(REFERER)
                    .and_then(|h| h.to_str().ok())
                    .and_then(|r| {
                        let uri = r.parse::<http::Uri>().ok()?;
                        uri.query().and_then(|q| parse_key_value_pair(q, name, "&"))
                    })
                {
                    return Some(val);
                }
            }
        }
        mock_state::QUERY_PARAMS.with(|q| q.borrow().get(name).cloned())
    }
}

#[cfg(any(target_arch = "wasm32", feature = "ssr"))]
fn parse_key_value_pair(s: &str, name: &str, sep: &str) -> Option<String> {
    s.split(sep).find_map(|s| {
        let mut parts = s.splitn(2, '=');
        let k = parts.next()?.trim();
        let v = parts.next()?.trim();
        if k == name { Some(v.to_string()) } else { None }
    })
}

/// Sets a cookie on both server and client.
///
/// - **SSR:** Inserts a `SET-COOKIE` header into `leptos_axum::ResponseOptions` (requires server setup with `leptos_routes_with_context`).
/// - **Client:** Updates `document.cookie`.
///
/// `options` should be a string like `; path=/; SameSite=Lax`.
pub fn set_cookie(name: &str, value: &str, options: &str) {
    #[cfg(all(target_arch = "wasm32", not(feature = "ssr")))]
    {
        let cookie = format!("{}={}{}", name, value, options);
        let _ = js_sys::Reflect::set(
            &document(),
            &wasm_bindgen::JsValue::from_str("cookie"),
            &wasm_bindgen::JsValue::from_str(&cookie),
        );
    }
    #[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
    {
        #[cfg(feature = "ssr")]
        {
            use http::HeaderValue;
            use http::header::SET_COOKIE;
            use leptos::prelude::use_context;
            use leptos_axum::ResponseOptions;

            if let Some(res) = use_context::<ResponseOptions>() {
                let cookie = format!("{}={}{}", name, value, options);
                if let Ok(val) = HeaderValue::from_str(&cookie) {
                    res.insert_header(SET_COOKIE, val);
                }
            }
        }
        mock_state::COOKIES.with(|c| {
            c.borrow_mut().insert(name.to_string(), value.to_string());
        });
        let _ = options;
    }
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
pub fn HydrateState<T>(#[prop(optional)] _marker: std::marker::PhantomData<T>) -> impl IntoView
where
    T: Hydratable + PartialEq,
{
    let (signal, resource) = use_hydrate_signal::<T>();
    provide_context(HydratedSignal(signal));
    provide_context(resource);

    #[cfg(any(feature = "ssr", target_arch = "wasm32"))]
    {
        let id = type_hydration_id::<T>();
        let script_id = format!("__lh_{}", id);
        view! {
            <script type="application/json" id={script_id}
                inner_html={
                    #[cfg(feature = "ssr")]
                    { serialize_for_injection(&T::initial()) }
                    #[cfg(not(feature = "ssr"))]
                    { "" }
                }
            />
        }
    }
    #[cfg(all(not(feature = "ssr"), not(target_arch = "wasm32")))]
    {
        view! {}
    }
}

/// Provides scoped hydrated state using the [`Hydratable`] trait.
///
/// Injects the server value and renders children inside the same component.
/// Use `use_hydrated::<T>()` in any child to access the signal.
#[component]
pub fn HydrateContext<T>(
    children: Children,
    #[prop(optional)] _marker: std::marker::PhantomData<T>,
) -> impl IntoView
where
    T: Hydratable + PartialEq,
{
    let (signal, resource) = use_hydrate_signal::<T>();
    provide_context(HydratedSignal(signal));
    provide_context(resource);
    view! {
        {children()}
        {
            #[cfg(any(feature = "ssr", target_arch = "wasm32"))]
            {
                let id = type_hydration_id::<T>();
                let script_id = format!("__lh_{}", id);
                view! {
                    <script type="application/json" id={script_id}
                        inner_html={
                            #[cfg(feature = "ssr")]
                            { serialize_for_injection(&T::initial()) }
                            #[cfg(not(feature = "ssr"))]
                            { "" }
                        }
                    />
                }
            }
            #[cfg(all(not(feature = "ssr"), not(target_arch = "wasm32")))]
            {
                view! { }
            }
        }
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
            "Hydrated LocalResource<{}> not found. Did you wrap this part of the tree in <HydrateState<{0}> />, <HydrateContext<{0}> />, <HydrateStateWith<{0}> />, or <HydrateContextWith<{0}> />?",
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
