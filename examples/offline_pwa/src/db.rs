#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use web_sys::{IdbDatabase, IdbTransactionMode, IdbOpenDbRequest, IdbRequest};

#[cfg(feature = "hydrate")]
pub async fn get_db() -> Result<IdbDatabase, String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let idb_factory = window.indexed_db().map_err(|e| format!("{:?}", e))?.ok_or_else(|| "no idb".to_string())?;
    let request = idb_factory.open_with_u32("offline_pwa_db", 1).map_err(|e| format!("{:?}", e))?;
    
    let (tx, rx) = futures::channel::oneshot::channel::<Result<IdbDatabase, String>>();
    let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

    {
        let tx_success = tx.clone();
        let on_success = Closure::once(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let request = target.dyn_into::<IdbOpenDbRequest>().unwrap();
            let db = request.result().unwrap().dyn_into::<IdbDatabase>().unwrap();
            if let Ok(mut guard) = tx_success.lock() {
                if let Some(t) = guard.take() {
                    let _ = t.send(Ok(db));
                }
            }
        });

        let on_upgrade_needed = Closure::once(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let request = target.dyn_into::<IdbOpenDbRequest>().unwrap();
            let db = request.result().unwrap().dyn_into::<IdbDatabase>().unwrap();
            let names = db.object_store_names();
            let mut found = false;
            for i in 0..names.length() {
                if names.get(i) == Some("state".to_string()) {
                    found = true;
                    break;
                }
            }
            if !found {
                let _ = db.create_object_store("state");
            }
        });

        let tx_error = tx.clone();
        let on_error = Closure::once(move |_: web_sys::Event| {
            if let Ok(mut guard) = tx_error.lock() {
                if let Some(t) = guard.take() {
                    let _ = t.send(Err("Failed to open IndexedDB".to_string()));
                }
            }
        });

        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onupgradeneeded(Some(on_upgrade_needed.as_ref().unchecked_ref()));
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        
        on_success.forget();
        on_upgrade_needed.forget();
        on_error.forget();
    }

    rx.await.map_err(|_| "channel closed".to_string())?
}

#[cfg(feature = "hydrate")]
pub async fn set_item(key: &str, value: &str) -> Result<(), String> {
    leptos::logging::log!("IDB: Setting {} = {}", key, value);
    let db = get_db().await?;
    let tx = db.transaction_with_str_and_mode("state", IdbTransactionMode::Readwrite).map_err(|e| format!("{:?}", e))?;
    let store = tx.object_store("state").map_err(|e| format!("{:?}", e))?;
    store.put_with_key(&JsValue::from_str(value), &JsValue::from_str(key)).map_err(|e| format!("{:?}", e))?;
    Ok(())
}

#[cfg(feature = "hydrate")]
pub async fn get_item(key: &str) -> Result<Option<String>, String> {
    leptos::logging::log!("IDB: Getting {}", key);
    let db = get_db().await?;
    let tx_db = db.transaction_with_str("state").map_err(|e| format!("{:?}", e))?;
    let store = tx_db.object_store("state").map_err(|e| format!("{:?}", e))?;
    let request = store.get(&JsValue::from_str(key)).map_err(|e| format!("{:?}", e))?;

    let (tx_res, rx_res) = futures::channel::oneshot::channel::<Result<Option<String>, String>>();
    let tx_res = std::sync::Arc::new(std::sync::Mutex::new(Some(tx_res)));

    {
        let tx_success = tx_res.clone();
        let on_success = Closure::once(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let request = target.dyn_into::<IdbRequest>().unwrap();
            let result = request.result().unwrap();
            if let Ok(mut guard) = tx_success.lock() {
                if let Some(t) = guard.take() {
                    let _ = t.send(Ok(result.as_string()));
                }
            }
        });

        let on_error = Closure::once(move |_: web_sys::Event| {
            if let Ok(mut guard) = tx_res.lock() {
                if let Some(t) = guard.take() {
                    let _ = t.send(Err("Failed to get item".to_string()));
                }
            }
        });

        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        
        on_success.forget();
        on_error.forget();
    }

    rx_res.await.map_err(|_| "channel closed".to_string())?
}

#[cfg(not(feature = "hydrate"))]
pub async fn get_item(_key: &str) -> Result<Option<String>, String> { Ok(None) }
#[cfg(not(feature = "hydrate"))]
pub async fn set_item(_key: &str, _value: &str) -> Result<(), String> { Ok(()) }
