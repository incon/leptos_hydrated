#[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
use leptos::prelude::*;
#[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
use std::collections::HashMap;

/// A request-scoped store for isomorphically shared hydration data.
/// This is best used for data you **have or can have on both sides** (e.g., cookies,
/// query params, or custom request state) to ensure immediate availability during
/// the hydration lifecycle.
///
/// This ensures that changes made during a request (such as setting cookies or
/// modifying shared state) are immediately visible to subsequent lookups in
/// the same request.
#[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
#[derive(Clone, Debug)]
struct HydrationStore {
    cookies: ArcRwSignal<HashMap<String, String>>,
    query: ArcRwSignal<HashMap<String, String>>,
}

#[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
impl HydrationStore {
    fn new() -> Self {
        #[cfg(feature = "ssr")]
        let mut cookies = HashMap::new();
        #[cfg(not(feature = "ssr"))]
        let cookies = HashMap::new();

        #[cfg(feature = "ssr")]
        let mut query = HashMap::new();
        #[cfg(not(feature = "ssr"))]
        let query = HashMap::new();

        #[cfg(feature = "ssr")]
        {
            use http::header::COOKIE;
            use http::request::Parts;

            if let Some(parts) = use_context::<Parts>() {
                // 1. Parse Cookies from request header
                if let Some(c_str) = parts.headers.get(COOKIE).and_then(|h| h.to_str().ok()) {
                    for part in c_str.split(';') {
                        let part = part.trim();
                        if let Some((k, v)) = part.split_once('=') {
                            cookies.insert(k.trim().to_string(), v.trim().to_string());
                        }
                    }
                }

                // 2. Parse Query from URI (matching axum::extract::Query logic)
                if let Some(q_str) = parts.uri.query() {
                    if let Ok(params) = serde_urlencoded::from_str::<Vec<(String, String)>>(q_str) {
                        query.extend(params);
                    }
                }
            }
        }

        Self {
            cookies: ArcRwSignal::new(cookies),
            query: ArcRwSignal::new(query),
        }
    }
}

/// Provides the hydration context for the current request.
/// Should be called inside the `leptos_routes_with_context` setup in `main.rs`.
#[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
pub fn provide_hydration_context() {
    provide_context(HydrationStore::new());
}

/// Internal helper to get the store, initializing it from context or creating a lazy one.
#[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
fn get_store() -> HydrationStore {
    use_context::<HydrationStore>().unwrap_or_else(|| {
        let store = HydrationStore::new();
        provide_context(store.clone());
        store
    })
}

/// Returns the value of a cookie by name.
///
/// Works isomorphically on both client and server.
/// On the server, it prioritizes cookies set during the current request.
pub fn get_cookie(name: &str) -> Option<String> {
    #[cfg(all(target_arch = "wasm32", not(feature = "ssr")))]
    {
        use leptos::prelude::document;
        let cookies = js_sys::Reflect::get(&document(), &wasm_bindgen::JsValue::from_str("cookie"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        for part in cookies.split(';') {
            let part = part.trim();
            if let Some((k, v)) = part.split_once('=') {
                if k.trim() == name {
                    return Some(v.trim().to_string());
                }
            }
        }
        None
    }

    #[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
    {
        get_store().cookies.read().get(name).cloned()
    }
}

/// Sets a cookie on both server and client.
///
/// - **SSR:** Updates the internal request store and appends a `SET-COOKIE` header to the response.
/// - **Client:** Updates `document.cookie`.
///
/// `options` should be a string like `; path=/; SameSite=Lax`.
pub fn set_cookie(name: &str, value: &str, options: &str) {
    #[cfg(all(target_arch = "wasm32", not(feature = "ssr")))]
    {
        use leptos::prelude::document;
        let cookie = format!("{}={}{}", name, value, options);
        let _ = js_sys::Reflect::set(
            &document(),
            &wasm_bindgen::JsValue::from_str("cookie"),
            &wasm_bindgen::JsValue::from_str(&cookie),
        );
    }

    #[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
    {
        let _ = options;
        // Update local store so subsequent reads in same request see it
        get_store().cookies.update(|c| {
            c.insert(name.to_string(), value.to_string());
        });

        #[cfg(feature = "ssr")]
        {
            use http::HeaderValue;
            use http::header::SET_COOKIE;
            use leptos_axum::ResponseOptions;

            if let Some(res) = use_context::<ResponseOptions>() {
                let cookie = format!("{}={}{}", name, value, options);
                if let Ok(val) = HeaderValue::from_str(&cookie) {
                    res.append_header(SET_COOKIE, val);
                }
            }
        }
    }
}

/// Returns the value of a query parameter by name.
///
/// Works isomorphically. On the server, it checks both the current URI and the Referer header.
pub fn get_query_param(name: &str) -> Option<String> {
    #[cfg(all(target_arch = "wasm32", not(feature = "ssr")))]
    {
        use leptos::prelude::window;
        let search = window().location().search().ok().unwrap_or_default();
        return web_sys::UrlSearchParams::new_with_str(&search)
            .ok()
            .and_then(|p| p.get(name));
    }

    #[cfg(any(feature = "ssr", not(target_arch = "wasm32")))]
    {
        // 1. Check current store (mocked or initialized from request)
        if let Some(val) = get_store().query.read().get(name).cloned() {
            return Some(val);
        }

        #[cfg(feature = "ssr")]
        {
            use http::header::REFERER;
            use http::request::Parts;

            // 2. Fallback to Referer header if present
            if let Some(parts) = use_context::<Parts>() {
                if let Some(val) = parts
                    .headers
                    .get(REFERER)
                    .and_then(|h| h.to_str().ok())
                    .and_then(|r| {
                        let uri = r.parse::<http::Uri>().ok()?;
                        let q_str = uri.query()?;
                        let params =
                            serde_urlencoded::from_str::<Vec<(String, String)>>(q_str).ok()?;
                        params.into_iter().find(|(k, _)| k == name).map(|(_, v)| v)
                    })
                {
                    return Some(val);
                }
            }
        }
        None
    }
}
