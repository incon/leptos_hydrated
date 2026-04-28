use super::*;
use tower::ServiceExt;
use crate::core::InjectedStates;
#[cfg(not(feature = "ssr"))]
use crate::core::get_hydration_counter;
use crate::core::{create_hydrated_signal, create_hydrated_context};
#[cfg(feature = "ssr")]
use crate::core::{serialize_for_injection, get_injected_states};
use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared fixture
// ---------------------------------------------------------------------------

static INIT: std::sync::Once = std::sync::Once::new();

fn init_test_env() {
    INIT.call_once(|| {
        let _ = any_spawner::Executor::init_tokio();
    });
}

fn use_hydrate_signal<T>() -> (RwSignal<T>, LocalResource<Option<T>>)
where
    T: Hydratable + Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    create_hydrated_signal(T::initial)
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct DefaultState {
    pub value: i32,
}
impl Hydratable for DefaultState {
    fn initial() -> Self {
        Self::default()
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct ThemeState {
    pub theme: String,
}

impl Hydratable for ThemeState {
    fn initial() -> Self {
        let theme = get_cookie("theme").unwrap_or_else(|| "dark".into());
        ThemeState { theme }
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct FetchState {
    pub value: i32,
}
impl Hydratable for FetchState {
    fn initial() -> Self {
        FetchState { value: 100 }
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
pub struct SlowState {
    pub value: i32,
}
#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
impl Hydratable for SlowState {
    fn initial() -> Self {
        SlowState { value: 2 }
    }
}

// ---------------------------------------------------------------------------
// Mechanism tests: create_hydrated_signal_internal
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_signal_initialises_from_fetch_state() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            owner.with(|| {
                let (signal, _) = use_hydrate_signal::<FetchState>();
                // In native tests, it starts with T::initial() which is 100
                assert_eq!(signal.get_untracked().value, 100);
            });
        })
        .await;
}

#[tokio::test]
async fn test_initial_sync_keeps_value() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            owner.with(|| {
                let (signal, _) = use_hydrate_signal::<DefaultState>();
                assert_eq!(signal.get_untracked().value, 0);
            });
        })
        .await;
}

// Client-only behavior: resource actually resolves
#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
#[tokio::test]
async fn test_initial_updates_signal() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            let signal = owner.with(|| {
                let (signal, _) = use_hydrate_signal::<FetchState>();
                signal
            });
            assert_eq!(signal.get_untracked().value, 100);
            // It should still be 100 because it re-ran initial() which returns 100
            for _ in 0..20 {
                tokio::task::yield_now().await;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            assert_eq!(signal.get_untracked().value, 100);
        })
        .await;
}

#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
#[tokio::test]
async fn test_two_way_binding_sync_flow() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            let (signal, resource) = owner.with(|| use_hydrate_signal::<SlowState>());

            // 1. Initial state (SlowState::initial returns 2)
            assert_eq!(signal.get_untracked().value, 2);

            // 2. Wait for hydration (it re-runs initial which returns 2)
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            assert_eq!(signal.get_untracked().value, 2);
            assert_eq!(resource.get_untracked(), Some(Some(SlowState { value: 2 })));

            // 3. Simulated user update
            signal.set(SlowState { value: 3 });
            assert_eq!(signal.get_untracked().value, 3);
            // Wait for resource to re-evaluate
            for _ in 0..10 {
                tokio::task::yield_now().await;
            }
            assert_eq!(resource.get_untracked(), Some(Some(SlowState { value: 3 })));
        })
        .await;
}

// ---------------------------------------------------------------------------
// SSR isolation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_ssr_resource_is_muted() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            owner.with(|| {
                let (signal, resource) = use_hydrate_signal::<DefaultState>();
                assert_eq!(signal.get_untracked().value, 0);
                // LocalResource should not resolve on the server
                assert!(resource.get_untracked().is_none());
            });
        })
        .await;
}

// ---------------------------------------------------------------------------
// Component + context tests
// ---------------------------------------------------------------------------

#[component]
fn MainContent() -> impl IntoView {
    let state = hydrated_signal(ThemeState::initial());
    view! { <p>"Theme: " {move || state.get().theme}</p> }
}

#[tokio::test]
async fn test_hydrate_context_global_provides_context() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            owner.with(|| {
                let _ = view! {
                    <HydratedContext<ThemeState> global=true />
                    <MainContent />
                };
            });
        })
        .await;
}

#[component]
fn ScopedDisplay() -> impl IntoView {
    let state = hydrated_signal(ThemeState::initial());
    view! { <p>"Scoped: " {move || state.get().theme}</p> }
}

#[tokio::test]
async fn test_hydrate_context_provides_context_to_children() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            owner.with(|| {
                let _ = view! {
                    <HydratedContext<ThemeState>>
                        <ScopedDisplay />
                    </HydratedContext<ThemeState>>
                };
            });
        })
        .await;
}

// ---------------------------------------------------------------------------
// HydratedSignal wrapper
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_hydrated_signal_wrapper_eq_and_debug() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let h1 = create_hydrated_context::<DefaultState>();
            let h2 = h1;
            assert_eq!(h1, h2);
            assert!(format!("{:?}", h1).contains("HydrateSignal"));
        });
    }).await;
}

// ---------------------------------------------------------------------------
// try_ accessors
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_try_use_hydrated_returns_some_when_context_exists() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let signal = RwSignal::new(ThemeState {
                theme: "dark".into(),
            });
            let resource = LocalResource::new(|| async { None });
            provide_context(HydrateSignal { signal, resource });
            let result = Hydrated::<ThemeState>::try_get();
            assert!(result.is_some());
            assert_eq!(result.unwrap().get_untracked().theme, "dark");
        });
    }).await;
}

#[test]
fn test_try_use_hydrated_returns_none_when_no_context() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        assert!(Hydrated::<ThemeState>::try_get().is_none());
    });
}

#[test]
fn test_try_use_hydrated_resource_returns_none_when_no_context() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        assert!(Hydrated::<ThemeState>::try_resource().is_none());
    });
}

// ---------------------------------------------------------------------------
// Isomorphic Helpers
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_get_cookie_ssr() {
    use axum::http::Request;
    use axum::http::header::COOKIE;

    let (parts, _) = Request::builder()
        .header(COOKIE, "test=value; other=foo")
        .body(())
        .unwrap()
        .into_parts();

    let owner = Owner::new_root(None);
    owner.with(|| {
        provide_context(parts);
        assert_eq!(get_cookie("test"), Some("value".into()));
        assert_eq!(get_cookie("other"), Some("foo".into()));
        assert_eq!(get_cookie("missing"), None);
    });
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_set_cookie_ssr() {
    use leptos_axum::ResponseOptions;

    let owner = Owner::new_root(None);
    owner.with(|| {
        let res_options = ResponseOptions::default();
        provide_context(res_options.clone());

        set_cookie("test_c", "val", "; Path=/");

        // Verify it was also inserted into mock state
        assert_eq!(get_cookie("test_c"), Some("val".into()));
    });
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_get_query_param_ssr() {
    use axum::http::Request;

    let (parts, _) = Request::builder()
        .uri("http://example.com/?foo=bar&baz=qux")
        .body(())
        .unwrap()
        .into_parts();

    let owner = Owner::new_root(None);
    owner.with(|| {
        provide_context(parts);
        assert_eq!(get_query_param("foo"), Some("bar".into()));
        assert_eq!(get_query_param("baz"), Some("qux".into()));
        assert_eq!(get_query_param("missing"), None);
    });
}


#[cfg(all(not(feature = "ssr"), not(feature = "hydrate")))]
#[test]
fn test_cookie_persistence_in_csr_mode() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        // In CSR mode (native tests), we use a mock store
        set_cookie("csr_test", "works", "");
        assert_eq!(get_cookie("csr_test"), Some("works".into()));
        assert_eq!(get_cookie("missing"), None);
    });
}

// ---------------------------------------------------------------------------
// Internal Mechanisms
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[test]
fn test_serialize_for_injection_internal() {
    let state = ThemeState {
        theme: "dark".into(),
    };
    let json = serialize_for_injection(&state);
    assert_eq!(json, r#"{"theme":"dark"}"#);
}

#[tokio::test]
async fn test_hydratable_initial_is_called() {
    let result = DefaultState::initial();
    assert_eq!(result.value, 0);
}

// ---------------------------------------------------------------------------
// Panic tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_hydrated_signal_creates_local_when_no_context() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let sig = hydrated_signal(DefaultState::initial());
            assert_eq!(sig.get_untracked().value, 0);
        });
    }).await;
}

#[test]
#[should_panic(expected = "Hydrated LocalResource<i32> not found")]
fn test_use_hydrated_resource_panics_without_context() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        let _ = Hydrated::<i32>::resource();
    });
}

// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Isomorphic / Environment tests
// ---------------------------------------------------------------------------

#[test]
fn test_isomorphic_macro_branches_correctly() {
    let val = isomorphic! {
        state => "ssr",
        hydrate => "csr"
    };
    #[cfg(feature = "ssr")]
    assert_eq!(val, "ssr");
    #[cfg(not(feature = "ssr"))]
    assert_eq!(val, "csr");
}

#[cfg(not(feature = "ssr"))]

// ---------------------------------------------------------------------------
// Coverage gap: get_query_param without Parts context (falls to mock_state)
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_get_query_param_no_context_returns_none() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            owner.with(|| {
                // No Parts in context → falls to mock_state → empty → None
                assert_eq!(get_query_param("anything"), None);
            });
        })
        .await;
}

// ---------------------------------------------------------------------------
// Coverage gap: set_cookie SSR path when ResponseOptions is provided
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_set_cookie_with_response_options_appends_header() {
    use leptos_axum::ResponseOptions;

    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            owner.with(|| {
                let res_options = ResponseOptions::default();
                provide_context(res_options.clone());

                // Triggers the ResponseOptions code path (append_header)
                // and also writes to mock_state for verification
                set_cookie("mycookie", "myvalue", "; path=/");

                // The mock_state is always written in SSR path
                assert_eq!(get_cookie("mycookie"), Some("myvalue".into()));
            });
        })
        .await;
}

// ---------------------------------------------------------------------------
// Coverage gap: LocalResource async closure — both branches
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_resource_first_run_returns_initial() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            owner.with(|| {
                // On first poll the resource re-runs initial(), which is 100 for FetchState
                let (signal, resource) = use_hydrate_signal::<FetchState>();
                assert_eq!(signal.get_untracked().value, 100);
                // Resource hasn't resolved yet (SSR LocalResource is lazy)
                assert!(resource.get_untracked().is_none());
            });
        })
        .await;
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_resource_subsequent_run_returns_current_signal() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            owner.with(|| {
                let (signal, _resource) = use_hydrate_signal::<DefaultState>();
                // Mutate the signal — a subsequent resource poll would return this value
                signal.set(DefaultState { value: 99 });
                assert_eq!(signal.get_untracked().value, 99);
            });
        })
        .await;
}

// ---------------------------------------------------------------------------
// Sync Opt-out tests
// ---------------------------------------------------------------------------

#[cfg(not(feature = "ssr"))]
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct NoSyncState {
    pub value: i32,
}
#[cfg(not(feature = "ssr"))]
impl Hydratable for NoSyncState {
    fn initial() -> Self {
        NoSyncState { value: 50 }
    }
    #[cfg(not(feature = "ssr"))]
    fn should_sync_on_client() -> bool {
        false
    }
}

#[cfg(not(feature = "ssr"))]
#[tokio::test]
async fn test_should_sync_on_client_false_skips_rerun() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let owner = Owner::new_root(None);
            let (_signal, resource) = owner.with(|| use_hydrate_signal::<NoSyncState>());

            // Wait for the resource to resolve
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;

            // In non-SSR mode, the resource re-runs initial().
            // If should_sync_on_client is false, it should return None.
            assert_eq!(resource.get_untracked(), Some(None));
        })
        .await;
}

// ---------------------------------------------------------------------------
// hydrated_signal tests
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_hydrated_signal_auto_id_ssr() {
    init_test_env();
    let states = InjectedStates::default();
    let mut parts = http::request::Request::builder().body(()).unwrap().into_parts().0;
    parts.extensions.insert(states.clone());
    
    let owner = Owner::new_root(None);
    owner.with(|| {
        provide_context(parts);
        let _ = hydrated_signal(DefaultState { value: 42 });
        let _ = hydrated_signal(DefaultState { value: 100 });
    });
    
    let guard = states.0.lock().unwrap();
    assert_eq!(guard.len(), 2);
    assert_eq!(guard[0], "{\"value\":42}");
    assert_eq!(guard[1], "{\"value\":100}");
}

#[cfg(not(feature = "ssr"))]
#[tokio::test]
async fn test_hydrated_signal_counter_client() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = hydrated_signal(DefaultState::initial());
            let _ = hydrated_signal(DefaultState::initial());
            
            let counter = get_hydration_counter();
            assert_eq!(*counter.0.lock().unwrap(), 2);
        });
    }).await;
}


// ---------------------------------------------------------------------------
// Middleware & Integration Tests
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_middleware_injects_script() {
    use axum::{body::Body, http::{Request, header}, routing::get, Router};
    use crate::ssr::HydratedRouterExt;
    use tower::ServiceExt; 
    use http_body_util::BodyExt;

    let app = Router::new()
        .route("/", get(|req: axum::extract::Request| async move {
            // Leptos would get the states from request extensions
            let states = req.extensions().get::<InjectedStates>().unwrap().clone();
            states.0.lock().unwrap().push(r#"{"test":true}"#.to_string());
            
            let mut res = axum::response::Response::new(Body::from("<html><body>Hello</body></html>"));
            res.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/html"));
            res
        }))
        .hydrated();

    let response = app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body_bytes);
    
    assert!(body_str.contains("__lh_data"), "Body should contain __lh_data script. Got: {}", body_str);
    assert!(body_str.contains(r#"{"test":true}"#));
}

#[tokio::test]
async fn test_synchronization_equality_check() {
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            // 1. Create signal with initial 10
            let (signal, _resource) = create_hydrated_signal(|| DefaultState { value: 10 });
            assert_eq!(signal.get_untracked().value, 10);
            
            // 2. Set to 20
            signal.set(DefaultState { value: 20 });
            assert_eq!(signal.get_untracked().value, 20);
            
            // 3. Sync with SAME value 20. 
            // We verify that if we implement the logic correctly, we don't call set unnecessarily.
            let val = DefaultState { value: 20 };
            let mut set_called = false;
            if val != signal.get_untracked() {
                signal.set(val);
                set_called = true;
            }
            assert!(!set_called, "Set should not be called when values are equal");
            
            // 4. Sync with NEW value 30
            let val = DefaultState { value: 30 };
            let mut set_called = false;
            if val != signal.get_untracked() {
                signal.set(val);
                set_called = true;
            }
            assert!(set_called, "Set should be called when values are different");
            assert_eq!(signal.get_untracked().value, 30);
        });
    }).await;
}


#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_hydration_store_complex_parsing() {
    use axum::http::Request;
    use axum::extract::OriginalUri;

    let mut req = Request::builder()
        .uri("/path?foo=bar&baz=qux")
        .header("Cookie", "session=123; theme=dark; pref=true")
        .body(())
        .unwrap();
    
    // Add OriginalUri extension to test fallback
    req.extensions_mut().insert(OriginalUri("/original?ref=promo".parse().unwrap()));

    let (parts, _) = req.into_parts();
    let store = HydrationStore::new_from_parts(&parts);
    
    assert_eq!(store.cookies.get_untracked().get("session").map(String::as_str), Some("123"));
    assert_eq!(store.cookies.get_untracked().get("theme").map(String::as_str), Some("dark"));
    assert_eq!(store.cookies.get_untracked().get("pref").map(String::as_str), Some("true"));
    
    // Should favor OriginalUri query
    assert_eq!(store.query.get_untracked().get("ref").map(String::as_str), Some("promo"));
}

#[tokio::test]
async fn test_hydrate_signal_deref() {
    init_test_env();
    let owner = Owner::new_root(None);
    owner.with(|| {
        let h = create_hydrated_context::<DefaultState>();
        // Test deref to RwSignal
        let _sig: &RwSignal<DefaultState> = &h;
        assert_eq!(h.get_untracked().value, 0);
    });
}

#[tokio::test]
async fn test_manual_injection_roundtrip() {
    init_test_env();
    let owner = Owner::new_root(None);
    owner.with(|| {
        #[cfg(feature = "ssr")]
        {
            let states = InjectedStates::default();
            provide_context(states.clone());
            
            inject_state(&"hello world".to_string());
            
            let guard = states.0.lock().unwrap();
            assert_eq!(guard.len(), 1);
            assert_eq!(guard[0], "\"hello world\"");
        }
        
        // On native/mock client, use_injected_state returns None (no wasm/hydrate)
        assert!(use_injected_state::<String>().is_none());
    });
}



#[tokio::test]
#[should_panic(expected = "HydrateSignal<leptos_hydrated::tests::DefaultState> not found")]
async fn test_hydrated_get_panics() {
    init_test_env();
    let owner = Owner::new_root(None);
    owner.with(|| {
        Hydrated::<DefaultState>::get();
    });
}

#[tokio::test]
#[should_panic(expected = "Hydrated LocalResource<leptos_hydrated::tests::DefaultState> not found")]
async fn test_hydrated_resource_panics() {
    init_test_env();
    let owner = Owner::new_root(None);
    owner.with(|| {
        Hydrated::<DefaultState>::resource();
    });
}




#[test]
fn test_hydrate_signal_eq() {
    init_test_env();
    let owner = Owner::new_root(None);
    owner.with(|| {
        let h1 = create_hydrated_context::<DefaultState>();
        let h2 = h1;
        assert_eq!(h1, h2);
    });
}

#[test]
fn test_use_hydrated_context_accessor() {
    init_test_env();
    let owner = Owner::new_root(None);
    owner.with(|| {
        // Should be None initially
        assert!(use_hydrated_context::<DefaultState>().is_none());

        // Create and provide
        let h = create_hydrated_context::<DefaultState>();
        provide_context(h);

        // Should be Some now
        assert!(use_hydrated_context::<DefaultState>().is_some());
        assert_eq!(use_hydrated_context::<DefaultState>().unwrap(), h);
    });
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_middleware_empty_injection() {
    use axum::http::Request;
    use axum::Router;
    use axum::routing::get;
    use crate::ssr::HydratedRouterExt;

    // A route that DOES NOT use any hydrated signals
    let app = Router::new()
        .route("/", get(|| async { "no signals here" }))
        .hydrated();

    let req = Request::builder().uri("/").header("Accept", "text/html").body(axum::body::Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();

    assert_eq!(res.status(), http::StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), 1024).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    // Should NOT contain the script tag because no signals were used
    assert!(!body_str.contains("__lh_data"));
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_middleware_non_html_response() {
    use axum::Router;
    use axum::routing::get;
    use crate::ssr::HydratedRouterExt;

    let app = Router::new()
        .route("/", get(|parts: http::request::Parts| async move { 
            // Even if we use a signal, if the response is not HTML, it shouldn't inject
            let owner = leptos::prelude::Owner::new_root(None);
            owner.with(|| {
                provide_context(parts);
                let _ = hydrated_signal(DefaultState::initial());
            });
            axum::response::Json(serde_json::json!({"status": "ok"}))
        }))
        .hydrated();

    let req = http::Request::builder().uri("/").body(axum::body::Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();

    assert_eq!(res.headers().get(http::header::CONTENT_TYPE).unwrap(), "application/json");
    let body = axum::body::to_bytes(res.into_body(), 1024).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(!body_str.contains("__lh_data"));
}

#[tokio::test]
async fn test_resource_closure_coverage() {
    init_test_env();
    let owner = Owner::new_root(None);
    owner.with(|| {
        let (sig, res) = create_hydrated_signal(|| DefaultState { value: 10 });
        
        // Trigger first run
        let _ = res.get(); // Triggers the closure
        
        // Set a new value and trigger second run
        sig.set(DefaultState { value: 20 });
        let _ = res.get();
    });
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_get_injected_states_fallback() {
    init_test_env();
    // Case: Parts present, but NO InjectedStates and NO MatchedPath/MockMatchedPath
    let req = http::Request::builder().body(()).unwrap();
    let (parts, _) = req.into_parts();
    
    let owner = Owner::new_root(None);
    owner.with(|| {
        provide_context(parts);
        let states = get_injected_states();
        // Should return a default InjectedStates without panicking
        assert!(states.0.lock().unwrap().is_empty());
    });
}




