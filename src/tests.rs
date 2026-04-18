use super::*;
use leptos::reactive::owner::Owner;
use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct ThemeState {
    pub theme: String,
}

#[tokio::test]
async fn test_use_hydrate_signal_csr_init() {
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
async fn test_use_hydrate_signal_resolution() {
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

        // Yield multiple times to ensure the spawn_local task in lib.rs runs.
        for _ in 0..20 {
            tokio::task::yield_now().await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        assert_eq!(signal.get(), 100);
    }).await;
}

#[cfg(not(feature = "ssr"))]
#[tokio::test]
async fn test_use_hydrate_signal_error_fallback() {
    let _ = any_spawner::Executor::init_tokio();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        let signal = owner.with(|| {
            let (signal, _resource) = use_hydrate_signal(
                || 42,
                || async {
                    Err::<i32, ServerFnError>(ServerFnError::Args("test error".to_string()))
                },
            );
            signal
        });
        
        assert_eq!(signal.get(), 42);
        
        for _ in 0..20 {
            tokio::task::yield_now().await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        assert_eq!(signal.get(), 0);
    }).await;
}

#[tokio::test]
async fn test_readme_examples() {
    let _ = any_spawner::Executor::init_tokio();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            // Test Hydrate Example (Global)
            // Values match to demonstrate flicker-free transition
            let _ = view! {
                <Hydrate
                    ssr_value=|| ThemeState { theme: "dark".into() }
                    fetcher=|| async { Ok(ThemeState { theme: "dark".into() }) }
                />
                {move || {
                    let state = use_hydrated::<ThemeState>();
                    assert_eq!(state.get().theme, "dark");
                    "".into_view()
                }}
            };

            // Test HydrateContext Example (Scoped)
            // Values match to demonstrate flicker-free transition
            let _ = view! {
                <HydrateContext
                    ssr_value=|| ThemeState { theme: "dark".into() }
                    fetcher=|| async { Ok(ThemeState { theme: "dark".into() }) }
                >
                    {move || {
                        let state = use_hydrated::<ThemeState>();
                        assert_eq!(state.get().theme, "dark");
                        "".into_view()
                    }}
                </HydrateContext>
            };
        });
    }).await;
}

#[test]
#[should_panic(expected = "HydratedSignal not found")]
fn test_use_hydrated_panic() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        let _ = use_hydrated::<i32>();
    });
}

#[test]
#[should_panic(expected = "Hydrated Resource not found")]
fn test_use_hydrated_resource_panic() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        let _ = use_hydrated_resource::<i32>();
    });
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn test_ssr_path() {
    let _ = any_spawner::Executor::init_tokio();
    let owner = Owner::new_root(None);
    owner.with(|| {
        let (signal, resource) = use_hydrate_signal(
            || 42,
            || async { Ok(100) },
        );
        assert_eq!(signal.get(), 42);
        // LocalResource should not resolve on the server
        assert!(resource.get().is_none());
    });
}

#[tokio::test]
async fn test_use_hydrated_resource_success() {
    let _ = any_spawner::Executor::init_tokio();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                <HydrateContext
                    ssr_value=|| 42
                    fetcher=|| async { Ok(100) }
                >
                    {move || {
                        let resource = use_hydrated_resource::<i32>();
                        let _ = resource.get();
                        "".into_view()
                    }}
                </HydrateContext>
            };
        });
    }).await;
}

#[test]
fn test_hydrated_signal_traits() {
    let owner = Owner::new_root(None);
    owner.with(|| {
        let signal = RwSignal::new(42);
        let h1 = HydratedSignal(signal);
        let h2 = h1; // Clone/Copy
        assert_eq!(h1, h2); // PartialEq/Eq
        assert!(format!("{:?}", h1).contains("HydratedSignal")); // Debug
    });
}
