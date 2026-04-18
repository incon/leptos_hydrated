use super::*;
use leptos::reactive::owner::Owner;

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
async fn test_hydrate_context_and_retrieval() {
    let _ = any_spawner::Executor::init_tokio();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                <HydrateContext
                    ssr_value=|| "initial".to_string()
                    fetcher=|| async { Ok("fetched".to_string()) }
                >
                    {move || {
                        let signal = use_hydrated::<String>();
                        let _resource = use_hydrated_resource::<String>();
                        assert_eq!(signal.get(), "initial");
                        "".into_view()
                    }}
                </HydrateContext>
            };
        });
    }).await;
}

#[tokio::test]
async fn test_hydrate_component() {
    let _ = any_spawner::Executor::init_tokio();
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let owner = Owner::new_root(None);
        owner.with(|| {
            let _ = view! {
                <Hydrate
                    ssr_value=|| 1.0
                    fetcher=|| async { Ok(2.0) }
                />
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
        assert!(resource.get().is_some());
    });
}
