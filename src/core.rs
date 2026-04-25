use crate::traits::Hydratable;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use serde::Serialize;
#[cfg(not(feature = "ssr"))]
use serde::de::DeserializeOwned;

/// A wrapper for a hydrated global signal provided via context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HydratedSignal<T: 'static>(pub RwSignal<T>);

pub(crate) fn type_hydration_id<T: 'static>() -> String {
    std::any::type_name::<T>()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
}

#[cfg(feature = "ssr")]
pub(crate) fn serialize_for_injection<T: Serialize>(value: &T) -> String {
    leptos::serde_json::to_string(value).unwrap_or_default()
}

#[cfg(not(feature = "ssr"))]
pub(crate) fn read_injected_state<T: DeserializeOwned>(id: &str) -> Option<T> {
    #[cfg(all(target_arch = "wasm32", feature = "hydrate"))]
    {
        use js_sys::JSON;
        use wasm_bindgen::JsCast as _;
        use wasm_bindgen::JsValue;

        let doc = document();
        let script_id = format!("__lh_{}", id);

        let el: JsValue = js_sys::Reflect::get(&doc, &JsValue::from_str("getElementById"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            .and_then(|f| f.call1(&doc, &JsValue::from_str(&script_id)).ok())
            .filter(|v: &JsValue| !v.is_null() && !v.is_undefined())?;

        let text = js_sys::Reflect::get(&el, &JsValue::from_str("textContent"))
            .ok()
            .and_then(|v| v.as_string())?;

        let js_val = JSON::parse(&text).ok()?;
        serde_wasm_bindgen::from_value(js_val).ok()
    }

    #[cfg(any(not(target_arch = "wasm32"), not(feature = "hydrate")))]
    {
        let _ = id;
        None
    }
}

/// Returns the value that was injected by the server for a specific type.
///
/// This is useful on the client inside `initial()` to merge server state with
/// local state (like localStorage).
#[cfg(not(feature = "ssr"))]
#[allow(dead_code)]
pub fn get_injected_state<T>() -> Option<T>
where
    T: crate::traits::Hydratable,
{
    let id = type_hydration_id::<T>();
    read_injected_state::<T>(&id)
}

/// The core hook for creating a hydrated signal.
///
/// This hook automatically manages signal hydration from a `LocalResource`
/// that calls `T::initial()`.
///
/// Returns `(RwSignal<T>, LocalResource<Option<T>>)`
pub fn use_hydrate_signal<T>() -> (RwSignal<T>, LocalResource<Option<T>>)
where
    T: Hydratable + PartialEq,
{
    #[cfg(not(feature = "ssr"))]
    let (initial_val, _is_injected) = {
        let id = type_hydration_id::<T>();
        let injected = read_injected_state::<T>(&id);
        let is_inj = injected.is_some();
        let val = injected.unwrap_or_else(T::initial);
        (val, is_inj)
    };

    #[cfg(feature = "ssr")]
    let (initial_val, _is_injected) = (T::initial(), false);

    let signal = RwSignal::new(initial_val.clone());
    let first_run = StoredValue::new(true);

    let resource = LocalResource::new(move || {
        let current_val = signal.get();
        let is_first = first_run.get_value();

        async move {
            if is_first {
                first_run.set_value(false);

                // On the client, check if we should skip the synchronization re-run.
                // This is crucial for HttpOnly cookies which are invisible to JS.
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
