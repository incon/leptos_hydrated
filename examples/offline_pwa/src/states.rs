use leptos_hydrated::*;
use serde::{Deserialize, Serialize};
use leptos_hydrated::hydrated;

#[cfg(not(feature = "ssr"))]
use leptos::prelude::*;

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
            ssr => Self::default(),
            csr => {
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

impl Hydratable for OnlineState {
    fn initial() -> Self {
        hydrated! {
            ssr => Self {
                online: get_cookie("online_status").map_or(true, |v| v == "true"),
            },
            csr => {
                let window = web_sys::window().unwrap();
                Self {
                    online: window.navigator().on_line(),
                }
            }
        }
    }

    #[cfg(not(feature = "ssr"))]
    fn on_hydrate(&self, online_state: RwSignal<Self>) {
        use leptos::ev;
        let window = web_sys::window().unwrap();
        let current_online = window.navigator().on_line();

        leptos::prelude::Effect::new(move |_| {
            leptos::logging::log!("OnlineState: Hydrated, setting up listeners...");

            // 1. Initial client-side sync
            if current_online != online_state.get_untracked().online {
                online_state.set(OnlineState {
                    online: current_online,
                });
                set_cookie(
                    "online_status",
                    if current_online { "true" } else { "false" },
                    "; path=/; max-age=31536000; SameSite=Lax",
                );
            }

            // 2. Setup permanent event listeners
            std::mem::forget(window_event_listener(ev::online, move |_| {
                leptos::logging::log!("OnlineState: online event");
                online_state.set(OnlineState { online: true });
                set_cookie(
                    "online_status",
                    "true",
                    "; path=/; max-age=31536000; SameSite=Lax",
                );
            }));
            std::mem::forget(window_event_listener(ev::offline, move |_| {
                leptos::logging::log!("OnlineState: offline event");
                online_state.set(OnlineState { online: false });
                set_cookie(
                    "online_status",
                    "false",
                    "; path=/; max-age=31536000; SameSite=Lax",
                );
            }));
        });
    }
}
