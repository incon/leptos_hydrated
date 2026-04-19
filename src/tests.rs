use super::*;
use leptos::reactive::owner::Owner;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared fixture
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Shared fixture
// ---------------------------------------------------------------------------

static INIT: std::sync::Once = std::sync::Once::new();

fn init_test_env() {
    INIT.call_once(|| {
        let _ = any_spawner::Executor::init_tokio();
    });
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct DefaultState {
    pub value: i32,
}
impl Hydratable for DefaultState {
    fn initial() -> Self { Self::default() }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct ThemeState {
    pub theme: String,
}

impl Hydratable for ThemeState {
    fn initial() -> Self {
        // Use isomorphic helpers to read from cookies/query params on both sides.
        let theme = get_cookie("theme").unwrap_or_else(|| "dark".into());
        ThemeState { theme }
    }

    fn fetch() -> impl std::future::Future<Output = Option<Self>> + Send + 'static {
        // Re-read from the same client-side source after hydration.
        async {
            let theme = get_cookie("theme").unwrap_or_else(|| "dark".into());
            Some(ThemeState { theme })
        }
    }
}

// ---------------------------------------------------------------------------
// Mechanism tests: use_hydrate_signal
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_signal_initialises_from_ssr_value() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let (signal, _) = use_hydrate_signal(
                || 42,
                || async { Some(100) },
            );
            assert_eq!(signal.get(), 42);
        });
    }).await;
}

#[tokio::test]
async fn test_fetch_none_keeps_initial_value() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let (signal, _) = use_hydrate_signal(
                || 42,
                || async { None::<i32> },
            );
            assert_eq!(signal.get(), 42);
        });
    }).await;
}

// CSR-only: resource actually resolves
#[cfg(not(feature = "ssr"))]
#[tokio::test]
async fn test_fetch_some_ok_updates_signal() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        let signal = owner.with(|| {
            let (signal, _) = use_hydrate_signal(
                || 42,
                || async { Some(100) },
            );
            signal
        });
        assert_eq!(signal.get(), 42);
        for _ in 0..20 { tokio::task::yield_now().await; }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(signal.get(), 100);
    }).await;
}

#[cfg(not(feature = "ssr"))]
#[tokio::test]
async fn test_fetch_some_err_keeps_initial_value() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        let signal = owner.with(|| {
            let (signal, _) = use_hydrate_signal(
                || 42,
                || async { None::<i32> },
            );
            signal
        });
        assert_eq!(signal.get(), 42);
        for _ in 0..20 { tokio::task::yield_now().await; }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(signal.get(), 42);
    }).await;
}
#[cfg(not(feature = "ssr"))]
#[tokio::test]
async fn test_two_way_binding_sync_flow() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        let (signal, resource) = owner.with(|| {
            use_hydrate_signal(
                || 1,
                || async { 
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    Some(2) 
                },
            )
        });

        // 1. Initial SSR state
        assert_eq!(signal.get(), 1);
        // LocalResource is always None on the server or initially on the client before task runs
        assert!(resource.get().is_none());

        // 2. Wait for hydration to finish
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(signal.get(), 2);
        assert_eq!(resource.get(), Some(Some(2)));

        // 3. Simulated user update (2-way binding concept)
        signal.set(3);
        assert_eq!(signal.get(), 3);
        // Wait for resource to re-evaluate (it tracks the signal now)
        for _ in 0..10 { tokio::task::yield_now().await; }
        assert_eq!(resource.get(), Some(Some(3)));
    }).await;
}

// ---------------------------------------------------------------------------
// SSR isolation
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_ssr_resource_is_muted() {
    init_test_env();
    let owner = Owner::new_root(None);
    owner.with(|| {
        let (signal, resource) = use_hydrate_signal(
            || 42,
            || async { panic!("Fetcher should not be called on SSR") },
        );
        assert_eq!(signal.get(), 42);
        // LocalResource should not resolve on the server
        assert!(resource.get().is_none());
    });
}

// ---------------------------------------------------------------------------
// Component + context tests
// ---------------------------------------------------------------------------

#[component]
fn MainContent() -> impl IntoView {
    let state = use_hydrated::<ThemeState>();
    view! { <p>"Theme: " {move || state.get().theme}</p> }
}

#[tokio::test]
async fn test_hydrate_state_provides_context() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                <HydrateState<ThemeState> />
                <MainContent />
            };
        });
    }).await;
}

#[component]
fn ScopedDisplay() -> impl IntoView {
    let state = use_hydrated::<ThemeState>();
    view! { <p>"Scoped: " {move || state.get().theme}</p> }
}

#[tokio::test]
async fn test_hydrate_context_provides_context_to_children() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                <HydrateContext<ThemeState>>
                    <ScopedDisplay />
                </HydrateContext<ThemeState>>
            };
        });
    }).await;
}

#[tokio::test]
async fn test_hydrate_state_with_provides_signal_and_resource() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                <HydrateStateWith
                    ssr_value=|| ThemeState { theme: "dark".into() }
                    fetcher=|| async { None::<ThemeState> }
                />
                {move || {
                    let signal = use_hydrated::<ThemeState>();
                    let _resource = use_hydrated_resource::<ThemeState>();
                    assert_eq!(signal.get_untracked().theme, "dark");
                    "".into_view()
                }}
            };
        });
    }).await;
}

#[tokio::test]
async fn test_hydrate_context_with_provides_signal_and_resource() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                <HydrateContextWith
                    ssr_value=|| ThemeState { theme: "dark".into() }
                    fetcher=|| async { None::<ThemeState> }
                >
                    {move || {
                        let signal = use_hydrated::<ThemeState>();
                        let _resource = use_hydrated_resource::<ThemeState>();
                        assert_eq!(signal.get_untracked().theme, "dark");
                        "".into_view()
                    }}
                </HydrateContextWith>
            };
        });
    }).await;
}

// ---------------------------------------------------------------------------
// HydratedSignal wrapper
// ---------------------------------------------------------------------------

#[test]
fn test_hydrated_signal_wrapper_eq_and_debug() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        let s = RwSignal::new(42);
        let h1 = HydratedSignal(s);
        let h2 = h1;
        assert_eq!(h1, h2);
        assert!(format!("{:?}", h1).contains("HydratedSignal"));
    });
}

// ---------------------------------------------------------------------------
// try_ accessors
// ---------------------------------------------------------------------------

#[test]
fn test_try_use_hydrated_returns_some_when_context_exists() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        provide_context(HydratedSignal(RwSignal::new(ThemeState { theme: "dark".into() })));
        let result = try_use_hydrated::<ThemeState>();
        assert!(result.is_some());
        assert_eq!(result.unwrap().get().theme, "dark");
    });
}

#[test]
fn test_try_use_hydrated_returns_none_when_no_context() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        assert!(try_use_hydrated::<ThemeState>().is_none());
    });
}

#[tokio::test]
async fn test_try_use_hydrated_resource_returns_some_when_context_exists() {
    init_test_env();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                <HydrateContextWith
                    ssr_value=|| ThemeState { theme: "dark".into() }
                    fetcher=|| async { None::<ThemeState> }
                >
                    {move || {
                        assert!(try_use_hydrated_resource::<ThemeState>().is_some());
                        "".into_view()
                    }}
                </HydrateContextWith>
            };
        });
    }).await;
}

#[test]
fn test_try_use_hydrated_resource_returns_none_when_no_context() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        assert!(try_use_hydrated_resource::<ThemeState>().is_none());
    });
}

// ---------------------------------------------------------------------------
// Isomorphic Helpers
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_get_cookie_ssr() {
    use http::Request;
    use http::header::COOKIE;
    
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
async fn test_get_query_param_ssr() {
    use http::Request;
    
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

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_get_header_ssr() {
    use http::Request;
    
    let (parts, _) = Request::builder()
        .header("X-Test", "value")
        .body(())
        .unwrap()
        .into_parts();
        
    let owner = Owner::new_root(None);
    owner.with(|| {
        provide_context(parts);
        assert_eq!(get_header("X-Test"), Some("value".into()));
        assert_eq!(get_header("Missing"), None);
    });
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_set_header_ssr() {
    use leptos_axum::ResponseOptions;
    
    let owner = Owner::new_root(None);
    owner.with(|| {
        let res = ResponseOptions::default();
        provide_context(res.clone());
        
        // These should not panic
        set_header("X-Response", "isomorphic");
        set_cookie("session", "abc", "; path=/");
    });
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_get_referer_query_param_ssr() {
    use http::Request;
    use http::header::REFERER;
    
    let (parts, _) = Request::builder()
        .header(REFERER, "http://site.com/page?ref=123")
        .body(())
        .unwrap()
        .into_parts();
        
    let owner = Owner::new_root(None);
    owner.with(|| {
        provide_context(parts);
        assert_eq!(get_referer_query_param("ref"), Some("123".into()));
        assert_eq!(get_referer_query_param("missing"), None);
    });
}

#[cfg(not(feature = "ssr"))]
#[test]
fn test_helpers_return_none_on_client() {
    // On client (without actual browser globals mocked) these should return None
    assert_eq!(get_cookie("any"), None);
    assert_eq!(get_query_param("any"), None);
    assert_eq!(get_header("any"), None);
    assert_eq!(get_referer_query_param("any"), None);
}

// ---------------------------------------------------------------------------
// Internal Mechanisms
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[test]
fn test_serialize_for_injection_internal() {
    let state = ThemeState { theme: "dark".into() };
    let json = serialize_for_injection(&state);
    assert_eq!(json, r#"{"theme":"dark"}"#);
    
    let id = type_hydration_id::<ThemeState>();
    assert!(id.contains("ThemeState"));
}

#[tokio::test]
async fn test_hydratable_default_fetch() {
    let result = DefaultState::fetch().await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_hydratable_custom_fetch() {
    let result = ThemeState::fetch().await;
    assert!(result.is_some());
}

// ---------------------------------------------------------------------------
// Panic tests
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "HydratedSignal<i32> not found")]
fn test_use_hydrated_panics_without_context() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        let _ = use_hydrated::<i32>();
    });
}

#[test]
#[should_panic(expected = "Hydrated LocalResource<i32> not found")]
fn test_use_hydrated_resource_panics_without_context() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        let _ = use_hydrated_resource::<i32>();
    });
}
