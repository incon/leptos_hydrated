use super::*;
use leptos::reactive::owner::Owner;
use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct ThemeState {
    pub theme: String,
}

impl Hydratable for ThemeState {
    fn initial() -> Self {
        // Read from cookie/URL synchronously
        ThemeState { theme: "dark".into() }
    }
    async fn fetch() -> Result<Self, ServerFnError> {
        // Use state already in the browser or refresh from API asynchronously
        Ok(ThemeState { theme: "light".into() })
    }
}

#[tokio::test]
async fn test_global_hydration_example() {
    let _ = any_spawner::Executor::init_tokio();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                // 1. Provide state anywhere in the tree
                <HydrateState<ThemeState> />

                // 2. Consume it anywhere in the tree
                <MainContent />
            };
        });
    }).await;
}

#[component]
fn MainContent() -> impl IntoView {
    let state = use_hydrated::<ThemeState>();
    view! { <p>"Theme: " {move || state.get().theme}</p> }
}

#[tokio::test]
async fn test_scoped_hydration_example() {
    let _ = any_spawner::Executor::init_tokio();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                // 1. Provide scoped state to a branch
                <HydrateContext<ThemeState>>
                    <ScopedThemeDisplay />
                </HydrateContext<ThemeState>>
            };
        });
    }).await;
}

#[component]
fn ScopedThemeDisplay() -> impl IntoView {
    let state = use_hydrated::<ThemeState>();
    view! { <p>"Scoped Theme: " {move || state.get().theme}</p> }
}

#[tokio::test]
async fn test_manual_hydration_example() {
    let _ = any_spawner::Executor::init_tokio();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                <HydrateStateWith
                    ssr_value=|| ThemeState { theme: "dark".into() }
                    fetcher=|| async { Ok(ThemeState { theme: "light".into() }) }
                />
                {move || {
                    let state = use_hydrated::<ThemeState>();
                    assert_eq!(state.get().theme, "dark");
                    "".into_view()
                }}
            };
        });
    }).await;
}

#[tokio::test]
async fn test_use_hydrate_signal_logic() {
    let _ = any_spawner::Executor::init_tokio();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let (signal, _resource) = use_hydrate_signal(
                || 42,
                || async {
                    tokio::task::yield_now().await;
                    Ok::<i32, ServerFnError>(100)
                },
            );
            assert_eq!(signal.get(), 42);
        });
    }).await;
}

#[cfg(not(feature = "ssr"))]
#[tokio::test]
async fn test_client_side_resolution() {
    let _ = any_spawner::Executor::init_tokio();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        
        let signal = owner.with(|| {
            let (signal, _resource) = use_hydrate_signal(
                || 42,
                || async {
                    Ok::<i32, ServerFnError>(100)
                },
            );
            signal
        });
        
        assert_eq!(signal.get(), 42);

        // Yield to allow background task to run
        for _ in 0..20 {
            tokio::task::yield_now().await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        assert_eq!(signal.get(), 100);
    }).await;
}

#[test]
#[should_panic(expected = "HydratedSignal not found. Did you wrap this part of the tree in <HydrateState />, <HydrateContext />, <HydrateStateWith />, or <HydrateContextWith />?")]
fn test_use_hydrated_panic() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        let _ = use_hydrated::<i32>();
    });
}

#[test]
#[should_panic(expected = "Hydrated Resource not found. Did you wrap this part of the tree in <HydrateContext /> or <HydrateContextWith />?")]
fn test_use_hydrated_resource_panic() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        let _ = use_hydrated_resource::<i32>();
    });
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_ssr_isolation() {
    let _ = any_spawner::Executor::init_tokio();
    let owner = Owner::new_root(None);
    owner.with(|| {
        let (signal, resource) = use_hydrate_signal(
            || 42,
            || async { Ok(100) },
        );
        assert_eq!(signal.get(), 42);
        // On SSR, resources don't resolve immediately in a synchronous test
        assert!(resource.get().is_none());
    });
}

#[test]
fn test_signal_wrappers() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        let signal = RwSignal::new(42);
        let h1 = HydratedSignal(signal);
        let h2 = h1; 
        assert_eq!(h1, h2);
        assert!(format!("{:?}", h1).contains("HydratedSignal"));
    });
}
