use crate::traits::Hydratable;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "ssr")]
use http::request::Parts;

/// Global shared state for injected scripts.
#[cfg(feature = "ssr")]
#[derive(Clone, Default, Debug)]
pub(crate) struct InjectedStates(pub Arc<Mutex<Vec<String>>>);

#[cfg(all(not(feature = "ssr"), target_arch = "wasm32"))]
use std::sync::OnceLock;

#[cfg(all(not(feature = "ssr"), target_arch = "wasm32"))]
static HYDRATION_DATA: OnceLock<Option<js_sys::Array>> = OnceLock::new();

#[cfg(all(not(feature = "ssr"), target_arch = "wasm32"))]
fn get_hydration_data() -> Option<&'static js_sys::Array> {
    HYDRATION_DATA.get_or_init(|| {
        let win = window();
        js_sys::Reflect::get(&win, &wasm_bindgen::JsValue::from_str("__lh_data"))
            .ok()
            .and_then(|v| {
                if v.is_undefined() || v.is_null() {
                    None
                } else {
                    use wasm_bindgen::JsCast;
                    v.dyn_into::<js_sys::Array>().ok()
                }
            })
    }).as_ref()
}

/// Accesses a hydrated signal of type `T` from the current context.
///
/// This is the primary way to share hydrated state between components.
///
/// # Panics
/// Panics in debug mode if the context is missing.
pub fn use_hydrated_context<T>() -> RwSignal<T>
where
    T: Hydratable + Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + PartialEq + 'static,
{
    use_context::<RwSignal<T>>().unwrap_or_else(|| {
        #[cfg(debug_assertions)]
        panic!(
            "\n\n[leptos_hydrated] MISSING CONTEXT PROVIDER\n\
            You are calling use_hydrated_context::<{}>() but no <HydratedContext<{0}>> was found in the parent tree.\n\n\
            FIX:\n\
            Wrap your component (or the whole app) in a provider:\n\
            <HydratedContext<{0}>>\n\
            \x20\x20\x20\x20<App />\n\
            </HydratedContext<{0}>>\n\n",
            std::any::type_name::<T>()
        );
        
        #[cfg(not(debug_assertions))]
        hydrated_signal(T::initial())
    })
}

/// Explicitly creates a new hydrated signal from `T::initial()`.
/// This is used by providers to ensure a fresh state is created for a new scope.
pub(crate) fn create_hydrated_context<T>() -> (RwSignal<T>, LocalResource<Option<T>>)
where
    T: Hydratable + Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + PartialEq + 'static,
{
    create_hydrated_signal(T::initial)
}


#[cfg(feature = "ssr")]
pub(crate) fn serialize_for_injection<T: serde::Serialize>(value: &T) -> String {
    leptos::serde_json::to_string(value).unwrap_or_default()
}

#[cfg(all(not(feature = "ssr"), target_arch = "wasm32"))]
pub(crate) fn read_injected_state<T: serde::de::DeserializeOwned>() -> Option<T> {
    let arr = get_hydration_data()?;
    let item = arr.shift();
    
    if item.is_null() || item.is_undefined() {
        return None;
    }

    serde_wasm_bindgen::from_value(item).ok()
}


#[cfg(feature = "ssr")]
pub(crate) fn get_injected_states() -> InjectedStates {
    if let Some(states) = use_context::<InjectedStates>() {
        states
    } else if let Some(parts) = use_context::<Parts>() {
        if let Some(states) = parts.extensions.get::<InjectedStates>() {
            provide_context(states.clone());
            states.clone()
        } else {
            #[cfg(feature = "ssr")]
            {
                // Only panic in debug mode if we are in a context that has been matched by Axum (a real request)
                // but the hydration marker is missing.
                #[cfg(debug_assertions)]
                {
                    use crate::ssr::HydrationMiddlewareMarker;
                    if parts.extensions.get::<axum::extract::MatchedPath>().is_some() 
                    && parts.extensions.get::<HydrationMiddlewareMarker>().is_none() 
                    {
                        panic!(
                            "\n\n[leptos_hydrated] MISSING MIDDLEWARE SETUP\n\
                            Hydrated signals require the `.hydrated()` middleware to be added to your Axum Router.\n\n\
                            FIX:\n\
                            use leptos_hydrated::HydratedRouterExt;\n\
                            let app = Router::new()\n\
                            \x20\x20\x20\x20.leptos_routes(...)\n\
                            \x20\x20\x20\x20.hydrated() // <--- Add this before .with_state()\n\
                            \x20\x20\x20\x20.with_state(leptos_options);\n\n"
                        );
                    }
                }
            }
            InjectedStates::default()
        }
    } else {
        InjectedStates::default()
    }
}



/// The core hook for creating a hydrated signal.
///
/// This hook automatically manages signal hydration from a `LocalResource`
/// that calls `T::initial()`.
///
/// Creates a new hydrated signal.
///
/// This is the primary entry point for hydrated state. Each call to this function
/// creates a new, independent signal. Synchronization between server and client
/// is handled automatically via a deterministic hydration counter.
pub fn hydrated_signal<T>(fallback: T) -> RwSignal<T>
where
    T: Hydratable + Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + PartialEq + 'static,
{
    create_hydrated_signal(|| fallback).0
}

/// The core hook for creating a hydrated signal.
///
/// This hook automatically manages signal hydration from a `LocalResource`
/// that calls `T::initial()`.
///
/// Returns `(RwSignal<T>, LocalResource<Option<T>>)`
pub(crate) fn create_hydrated_signal<T, F>(
    fallback: F,
) -> (RwSignal<T>, LocalResource<Option<T>>)
where
    T: Hydratable + Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
    F: FnOnce() -> T + 'static,
{
    #[cfg(all(not(feature = "ssr"), target_arch = "wasm32"))]
    let initial_val = {
        let injected = read_injected_state::<T>();
        injected.unwrap_or_else(fallback)
    };

    #[cfg(feature = "ssr")]
    let initial_val = {
        let val = fallback();
        let states = get_injected_states();
        if let Ok(mut states_guard) = states.0.lock() {
            let json = serialize_for_injection(&val);
            states_guard.push(json);
        }
        val
    };

    #[cfg(all(not(feature = "ssr"), not(target_arch = "wasm32")))]
    let initial_val = fallback();

    let signal = RwSignal::new(initial_val.clone());

    #[cfg(all(not(feature = "ssr"), target_arch = "wasm32"))]
    {
        // Provide the signal to the context so that on_hydrate can find it via use_hydrated_context
        provide_context(signal);
        initial_val.on_hydrate();
    }

    let first_run = StoredValue::new(true);

    let resource = LocalResource::new(move || {
        let current_val = signal.get();
        let is_first = first_run.get_value();

        async move {
            if is_first {
                first_run.set_value(false);

                // On the client, check if we should skip the synchronization re-run.
                #[cfg(all(not(feature = "ssr"), target_arch = "wasm32"))]
                if !T::should_sync_on_client() {
                    return None;
                }

                Some(T::initial())
            } else {
                Some(current_val)
            }
        }
    });

    #[cfg(all(not(feature = "ssr"), target_arch = "wasm32", not(test)))]
    {
        let resource_cloned = resource.clone();
        leptos::task::spawn_local(async move {
            if let Some(val) = resource_cloned.await {
                    signal.set(val);
                }
        });
    }

    (signal, resource)
}
