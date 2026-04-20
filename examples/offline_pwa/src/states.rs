use crate::db;
use leptos_hydrated::*;
use serde::{Deserialize, Serialize};

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
        // Initial state is empty on server (or you could use cookies)
        Self::default()
    }

    fn fetch() -> impl std::future::Future<Output = Option<Self>> + Send + 'static {
        // On client, try to restore from IndexedDB
        async {
            leptos::logging::log!("IDB: Fetching todos state...");
            match db::get_item("todos").await {
                Ok(Some(json)) => {
                    leptos::logging::log!("IDB: Fetched JSON: {}", json);
                    let js_val = js_sys::JSON::parse(&json).ok();
                    js_val.and_then(|v| serde_wasm_bindgen::from_value(v).ok())
                }
                Ok(None) => {
                    leptos::logging::log!("IDB: No todos found.");
                    None
                }
                Err(e) => {
                    leptos::logging::log!("IDB: Fetch error: {:?}", e);
                    None
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

impl Hydratable for OnlineState {
    fn initial() -> Self {
        Self { online: get_cookie("online_status").is_none_or(|v| v == "true") }
    }
}
