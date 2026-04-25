#![allow(unused_imports)]
use leptos::context::use_context;
use leptos::prelude::{RwSignal, Update};
use leptos_hydrated::hydrated;
use leptos_hydrated::*;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "ssr"))]
use leptos_use::use_event_listener;

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct TodoItem {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct TodoState {
    pub todos: Vec<TodoItem>,
}

impl Hydratable for TodoState {
    fn initial() -> Self {
        hydrated! {
            server => Self::default(),
            client => {
                // On client, try to restore from localStorage (sync)
                leptos::logging::log!("LocalStorage: Restoring todos state...");
                let window = web_sys::window().unwrap();
                let storage = window.local_storage().unwrap().unwrap();
                match storage.get_item("todos") {
                    Ok(Some(json)) => {
                        leptos::logging::log!("LocalStorage: Fetched JSON: {}", json);
                        js_sys::JSON::parse(&json)
                            .ok()
                            .and_then(|js_val| serde_wasm_bindgen::from_value(js_val).ok())
                            .unwrap_or_default()
                    }
                    _ => {
                        leptos::logging::log!("LocalStorage: No todos found.");
                        Self::default()
                    }
                }
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct OnlineState {
    pub online: bool,
}

impl Default for OnlineState {
    fn default() -> Self {
        Self { online: true }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PwaInit {
    pub was_hydrated: bool,
}

impl Hydratable for OnlineState {
    fn initial() -> Self {
        let was_hydrated = use_context::<PwaInit>()
            .map(|c| c.was_hydrated)
            .unwrap_or(true);
        Self {
            online: was_hydrated,
        }
    }

    #[cfg(not(feature = "ssr"))]
    fn on_hydrate(&self, online_state: RwSignal<Self>) {
        use leptos::ev;
        use leptos_use::use_event_listener;

        let _ = use_event_listener(web_sys::window(), ev::online, move |_| {
            online_state.update(|s| s.online = true);
        });

        let _ = use_event_listener(web_sys::window(), ev::offline, move |_| {
            online_state.update(|s| s.online = false);
        });
    }
}
