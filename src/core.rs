use crate::traits::Hydratable;
use leptos::prelude::*;
use std::sync::{Arc, Mutex};

#[cfg(feature = "ssr")]
use http::request::Parts;


/// Global shared state for injected scripts.
#[derive(Clone, Default, Debug)]
pub struct InjectedStates(pub Arc<Mutex<Vec<String>>>);

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

/// A wrapper for a hydrated global signal provided via context.
pub struct HydrateSignal<T: 'static> {
    /// The underlying reactive signal.
    pub signal: RwSignal<T>,
    /// The resource used for synchronization.
    pub resource: LocalResource<Option<T>>,
}

impl<T: 'static> std::fmt::Debug for HydrateSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HydrateSignal")
            .field("signal", &self.signal)
            .finish()
    }
}

impl<T: 'static> Clone for HydrateSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for HydrateSignal<T> {}

impl<T: 'static> PartialEq for HydrateSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.signal == other.signal
    }
}

impl<T: 'static> Eq for HydrateSignal<T> {}

/// Creates a new hydrated signal with an automatically generated ID.
/// State is injected by the `inject_logic` middleware on the server.
/// This also sets up automatic synchronization via a `LocalResource`.
pub fn use_hydrated_context<T>() -> HydrateSignal<T>
where
    T: Hydratable + Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    let (signal, resource) = create_hydrated_signal(T::initial);
    HydrateSignal { signal, resource }
}

impl<T: 'static> std::ops::Deref for HydrateSignal<T> {
    type Target = RwSignal<T>;

    fn deref(&self) -> &Self::Target {
        &self.signal
    }
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

        let el: JsValue = js_sys::Reflect::get(&doc, &JsValue::from_str("getElementById"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            .and_then(|f| f.call1(&doc, &JsValue::from_str(script_id)).ok())
            .filter(|v: &JsValue| !v.is_null() && !v.is_undefined())?;

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


/// The core hook for creating a hydrated signal.
///
/// This hook automatically manages signal hydration from a `LocalResource`
/// that calls `T::initial()`.
///
/// Creates a new hydrated signal or retrieves one from context.
///
/// This is the primary entry point for hydrated state. If a `HydrateSignal<T>`
/// is found in the current context (provided by `HydratedContext`), it will be returned.
/// Otherwise, a new hydrated signal is created with the provided fallback value.
pub fn hydrated_signal<T>(fallback: T) -> RwSignal<T>
where
    T: Hydratable + Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    if let Some(s) = use_context::<HydrateSignal<T>>() {
        s.signal
    } else {
        create_hydrated_signal(|| fallback).0
    }
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

        // Push to injected states to be picked up by the inject_logic middleware
        if let Some(parts) = leptos::prelude::use_context::<Parts>() {
            if let Some(states) = parts.extensions.get::<InjectedStates>() {
                if let Ok(mut states_guard) = states.0.lock() {
                    let json = serialize_for_injection(&val);
                    states_guard.push(json);
                }
            }
        }
        val
    };

    let signal = RwSignal::new(initial_val.clone());
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

    #[cfg(not(feature = "ssr"))]
    {
        initial_val.on_hydrate(signal);
    }

    (signal, resource)
}

