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
pub struct ThemeState {
    pub theme: String,
}

impl Hydratable for ThemeState {
    fn initial() -> Self {
        ThemeState { theme: "dark".into() }
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
                || async { Some(Ok::<i32, ServerFnError>(100)) },
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
                || async { None::<Result<i32, ServerFnError>> },
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
                || async { Some(Ok::<i32, ServerFnError>(100)) },
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
                || async {
                    Some(Err::<i32, ServerFnError>(ServerFnError::MissingArg(
                        "test".into(),
                    )))
                },
            );
            signal
        });
        assert_eq!(signal.get(), 42);
        for _ in 0..20 { tokio::task::yield_now().await; }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(signal.get(), 42);
    }).await;
}

// ---------------------------------------------------------------------------
// SSR isolation
// ---------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_ssr_resource_does_not_resolve_synchronously() {
    init_test_env();
    let owner = Owner::new_root(None);
    owner.with(|| {
        let (signal, resource) = use_hydrate_signal(
            || 42,
            || async { Some(Ok::<i32, ServerFnError>(100)) },
        );
        assert_eq!(signal.get(), 42);
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
                    fetcher=|| async { None::<Result<ThemeState, ServerFnError>> }
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
                    fetcher=|| async { None::<Result<ThemeState, ServerFnError>> }
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
                    fetcher=|| async { None::<Result<ThemeState, ServerFnError>> }
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
// Panic tests
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "HydratedSignal<i32> not found")]
fn test_use_hydrated_panics_without_context() {
    let owner = Owner::new_root(None);
    owner.with(|| { let _ = use_hydrated::<i32>(); });
}

#[test]
#[should_panic(expected = "Hydrated Resource<i32> not found")]
fn test_use_hydrated_resource_panics_without_context() {
    let owner = Owner::new_root(None);
    owner.with(|| { let _ = use_hydrated_resource::<i32>(); });
}
