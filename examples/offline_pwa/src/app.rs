#[cfg(not(feature = "ssr"))]
use crate::db;
use crate::states::*;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_hydrated::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    ParamSegment, StaticSegment,
    components::{A, Route, Router, Routes},
    hooks::use_params,
    params::Params,
};

pub fn get_version() -> String {
    use std::sync::OnceLock;
    static VERSION: OnceLock<String> = OnceLock::new();

    VERSION
        .get_or_init(|| {
            if cfg!(debug_assertions) {
                format!(
                    "dev-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                )
            } else {
                env!("CARGO_PKG_VERSION").to_string()
            }
        })
        .clone()
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();
    let version = get_version();

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="theme-color" content="#ffffff" />
                <Title text="Offline Todo"/>
                <link rel="icon" type="image/svg+xml" href=format!("/icon.svg?v={version}") />
                <link rel="manifest" href="/manifest.json" />
                <HydrationScripts options=options.clone() />
                <MetaTags/>
                <Stylesheet id="leptos" href="/pkg/offline_pwa.css"/>
                <script>
                    "if ('serviceWorker' in navigator) {
                        navigator.serviceWorker.register('/sw.js');
                    }
                    
                    // WebSocket interceptor to enable path-based hot reload through Caddy (single port setup)
                    (function() {
                        var OriginalWebSocket = window.WebSocket;
                        window.WebSocket = function(url, protocols) {
                            console.log('[Leptos Proxy] WebSocket request to:', url);
                            try {
                                var urlObj = new URL(url, window.location.origin);
                                // Identify the Leptos reload websocket (usually hits the root path on the reload port)
                                if (urlObj.pathname === '/' || urlObj.pathname === '' || url.includes('live_reload')) {
                                    var protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                                    url = protocol + '//' + window.location.host + '/live_reload';
                                    console.log('[Leptos Proxy] Rewriting to:', url);
                                }
                            } catch (e) {
                                console.error('[Leptos Proxy] Error:', e);
                            }
                            return new OriginalWebSocket(url, protocols);
                        };
                        window.WebSocket.prototype = OriginalWebSocket.prototype;
                    })();"
                </script>
                {
                    #[cfg(all(debug_assertions, feature = "ssr"))]
                    view! { <AutoReload options=options /> }
                }
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[derive(Copy, Clone)]
pub struct OnlineContext {
    pub online: RwSignal<bool>,
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let online = RwSignal::new(true);
    provide_context(OnlineContext { online });

    #[cfg(not(feature = "ssr"))]
    {
        use leptos::ev;

        // Ensure we check native state on boot if it happens to be accurate
        Effect::new(move |_| {
            let is_online = web_sys::window().unwrap().navigator().on_line();
            if !is_online {
                online.set(false);
            }
        });

        std::mem::forget(window_event_listener(ev::online, move |_| {
            leptos::logging::log!("App: browser went online");
            online.set(true);
        }));
        std::mem::forget(window_event_listener(ev::offline, move |_| {
            leptos::logging::log!("App: browser went offline");
            online.set(false);
        }));
    }

    view! {
        <Title text="Offline Todo"/>
        <div id="app-root">
            <OnlineStatus />
            <div class="app-container">
                <HydrateContext<TodoState>>
                    <TodoPersistence />
                    <Router>
                        <main>
                            <Routes fallback=|| "Page not found.".into_view()>
                                <Route path=StaticSegment("") view=TodoPage />
                                <Route path=(StaticSegment("todo"), ParamSegment("id")) view=TodoDetailsPage />
                            </Routes>
                        </main>
                    </Router>
                </HydrateContext<TodoState>>
            </div>
        </div>
    }
}

#[component]
fn OnlineStatus() -> impl IntoView {
    let online = use_context::<OnlineContext>()
        .map(|ctx| ctx.online)
        .unwrap_or_else(|| RwSignal::new(true));

    view! {
        <div id="online-status" class=move || format!("status-banner {}", if online.get() { "online" } else { "offline" })>
            {move || if online.get() {
                "● Online"
            } else {
                "○ Offline - Using local storage"
            }}
        </div>
    }
}

#[component]
fn TodoPersistence() -> impl IntoView {
    #[cfg(not(feature = "ssr"))]
    Effect::new(move |_| {
        let resource = use_hydrated_resource::<TodoState>();

        // Wait for hydration to finish before we start persisting changes
        if resource.get().is_none() {
            return;
        }

        let state = use_hydrated::<TodoState>();
        let current = state.get();

        leptos::task::spawn_local(async move {
            if let Ok(js_val) = serde_wasm_bindgen::to_value(&current) {
                if let Ok(json) = js_sys::JSON::stringify(&js_val) {
                    if let Some(json) = json.as_string() {
                        let _ = db::set_item("todos", &json).await;
                    }
                }
            }
        });
    });

    view! { "" }
}

#[component]
fn TodoPage() -> impl IntoView {
    let state = use_hydrated::<TodoState>();
    let new_todo = RwSignal::new(String::new());
    let new_description = RwSignal::new(String::new());

    let add_todo = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let title = new_todo.get_untracked();
        let description = new_description.get_untracked();
        if !title.is_empty() {
            state.update(|s| {
                let id = s.todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
                s.todos.push(TodoItem {
                    id,
                    title,
                    description,
                    completed: false,
                });
            });
            new_todo.set(String::new());
            new_description.set(String::new());
        }
    };

    view! {
        <div class="todo-page">
            <div class="header-section">
                <h1>"Offline Todo"</h1>
                <p class="subtitle">"State is persisted to IndexedDB and works offline via Service Workers."</p>
            </div>

            <div class="card form-card">
                <h2>"Add Todo"</h2>
                <form on:submit=add_todo class="todo-form">
                    <div class="form-field">
                        <label for="todo-title">"Task Title"</label>
                        <input
                            id="todo-title"
                            type="text"
                            placeholder="What needs to be done?"
                            prop:value=new_todo
                            on:input=move |ev| new_todo.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label for="todo-desc">"Description"</label>
                        <textarea
                            id="todo-desc"
                            placeholder="Add some details..."
                            rows="3"
                            prop:value=new_description
                            on:input=move |ev| new_description.set(event_target_value(&ev))
                        />
                    </div>
                    <button type="submit" class="submit-button">"Add Task"</button>
                </form>
            </div>

            <div class="card list-card">
                <h2>"Todos"</h2>
                <Show
                    when=move || !state.get().todos.is_empty()
                    fallback=|| view! {
                        <div class="empty-state">
                            <p>"You have no todos yet. Add one above!"</p>
                        </div>
                    }
                >
                    <ul class="todo-list">
                    <For
                        each=move || state.get().todos
                        key=|todo| todo.id
                        let:todo
                    >
                        <li class=move || if todo.completed { "completed" } else { "" }>
                            <input
                                type="checkbox"
                                prop:checked=todo.completed
                                on:change=move |_| {
                                    state.update(|s| {
                                        if let Some(t) = s.todos.iter_mut().find(|t| t.id == todo.id) {
                                            t.completed = !t.completed;
                                        }
                                    });
                                }
                            />
                            <A href=move || format!("/todo/{}", todo.id)>
                                <span>{todo.title}</span>
                            </A>
                            <button class="delete-btn" on:click=move |_| {
                                state.update(|s| {
                                    s.todos.retain(|t| t.id != todo.id);
                                });
                            }>"×"</button>
                        </li>
                    </For>
                    </ul>
                </Show>
            </div>
        </div>
    }
}

#[derive(Params, PartialEq, Debug, Clone)]
struct TodoParams {
    id: u64,
}

#[component]
fn TodoDetailsPage() -> impl IntoView {
    let params = use_params::<TodoParams>();
    let state = use_hydrated::<TodoState>();

    let todo = move || {
        params
            .get()
            .ok()
            .and_then(|p| state.get().todos.into_iter().find(|t| t.id == p.id))
    };

    view! {
        <div class="todo-details">
            <A href="/">"← Back to List"</A>
            {move || match todo() {
                Some(todo) => Either::Left(view! {
                    <div class="card">
                        <h1>{todo.title}</h1>
                        <p class="status">
                            {if todo.completed { "✅ Completed" } else { "⏳ In Progress" }}
                        </p>
                        <hr />
                        <div class="description">
                            <h3>"Description"</h3>
                            <p>{if todo.description.is_empty() { "No description provided.".to_string() } else { todo.description }}</p>
                        </div>
                    </div>
                }),
                None => Either::Right(view! {
                    <div class="error">
                        <h1>"Todo Not Found"</h1>
                        <p>"The todo with the requested ID does not exist."</p>
                    </div>
                })
            }}
        </div>
    }
}
