use crate::traits::Hydratable;
use leptos::prelude::*;
use std::sync::{Arc, Mutex};

#[cfg(feature = "ssr")]
use http::request::Parts;


/// Global shared state for injected scripts.
#[cfg(feature = "ssr")]
#[derive(Clone, Default, Debug)]
pub(crate) struct InjectedStates(pub Arc<Mutex<Vec<String>>>);

/// Counter for automatic hydration IDs on the client.
#[cfg(not(feature = "ssr"))]
#[derive(Clone, Default, Debug)]
pub(crate) struct HydrationCounter(pub Arc<Mutex<usize>>);

#[cfg(not(feature = "ssr"))]
impl HydrationCounter {
    pub fn next(&self) -> usize {
        let mut guard = self.0.lock().unwrap();
        let val = *guard;
        *guard += 1;
        val
    }
}

#[cfg(not(feature = "ssr"))]
pub(crate) fn get_hydration_counter() -> HydrationCounter {
    use_context::<HydrationCounter>().unwrap_or_else(|| {
        let counter = HydrationCounter::default();
        provide_context(counter.clone());
        counter
    })
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

#[cfg(not(feature = "ssr"))]
pub(crate) fn read_injected_state<T: serde::de::DeserializeOwned>(index: usize) -> Option<T> {
    #[cfg(all(target_arch = "wasm32", feature = "hydrate"))]
    {
        use js_sys::JSON;
        use wasm_bindgen::JsCast as _;
        use wasm_bindgen::JsValue;

        let doc = document();
        let script_id = "__lh_data";

        let el = doc.get_element_by_id(script_id);
        if el.is_none() {
            #[cfg(debug_assertions)]
            panic!(
                "\n\n[leptos_hydrated] MISSING HYDRATION SCRIPTS\n\
                You are using a hydrated signal but <HydrationScripts /> is missing from your HTML head.\n\n\
                FIX:\n\
                <head>\n\
                \x20\x20\x20\x20...\n\
                \x20\x20\x20\x20<HydrationScripts options=options />\n\
                </head>\n\n"
            );
            
            #[cfg(not(debug_assertions))]
            return None;
        }
        let el = el.unwrap();

        let text = js_sys::Reflect::get(&el, &JsValue::from_str("textContent"))
            .ok()
            .and_then(|v| v.as_string())?;

        let js_val = JSON::parse(&text).ok()?;
        let arr = js_val.dyn_into::<js_sys::Array>().ok()?;
        let item = arr.get(index as u32);
        
        if item.is_null() || item.is_undefined() {
            return None;
        }

        serde_wasm_bindgen::from_value(item).ok()
    }

    #[cfg(any(not(target_arch = "wasm32"), not(feature = "hydrate")))]
    {
        let _ = index;
        None
    }
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
                use crate::ssr::HydrationMiddlewareMarker;
                // Only panic in debug mode if we are in a context that has been matched by Axum (a real request)
                // but the hydration marker is missing.
                #[cfg(debug_assertions)]
                {
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


/// Reads the next available injected state from the server.
/// 
/// This is the client-side counterpart to `get_injected_states()`.
/// It increments the internal hydration counter.
pub fn use_injected_state<T>() -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    #[cfg(not(feature = "ssr"))]
    {
        let counter = get_hydration_counter();
        let index = counter.next();
        read_injected_state(index)
    }
    #[cfg(feature = "ssr")]
    {
        None
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
    #[cfg(not(feature = "ssr"))]
    let initial_val = {
        let counter = get_hydration_counter();
        let index = counter.next();
        let injected = read_injected_state::<T>(index);
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

    let signal = RwSignal::new(initial_val.clone());

    #[cfg(not(feature = "ssr"))]
    {
        initial_val.on_hydrate(signal);
    }

    let first_run = StoredValue::new(true);

    let resource = LocalResource::new(move || {
        let current_val = signal.get();
        let is_first = first_run.get_value();

        async move {
            if is_first {
                first_run.set_value(false);

                // On the client, check if we should skip the synchronization re-run.
                #[cfg(not(feature = "ssr"))]
                if !T::should_sync_on_client() {
                    return None;
                }

                Some(T::initial())
            } else {
                Some(current_val)
            }
        }
    });

    #[cfg(all(not(feature = "ssr"), not(test)))]
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



