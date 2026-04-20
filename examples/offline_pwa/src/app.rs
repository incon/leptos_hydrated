#[cfg(not(feature = "ssr"))]
use crate::db;
use crate::states::*;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_hydrated::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    hooks::use_params,
    params::Params,
    ParamSegment, StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="theme-color" content="#ffffff" />
                <Title text="Offline Todo"/>
                <link rel="icon" type="image/svg+xml" href="/icon.svg" />
                <link rel="manifest" href="/manifest.json" />
                <HydrationScripts options=options.clone() />
                <MetaTags/>
                <Stylesheet id="leptos" href="/pkg/offline_pwa.css"/>
                <script>
                    "if ('serviceWorker' in navigator) {
                        navigator.serviceWorker.register('/sw.js');
                    }"
                </script>
                <script>
                    "window.__INITIAL_ONLINE__ = navigator.onLine;"
                </script>
                <OnlineAutoReload options=options.clone() />
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

    let initial_online = {
        #[cfg(not(feature = "hydrate"))]
        { true }
        #[cfg(feature = "hydrate")]
        {
            js_sys::Reflect::get(&web_sys::window().unwrap(), &::wasm_bindgen::JsValue::from_str("__INITIAL_ONLINE__"))
                .and_then(|v| v.as_bool().ok_or(::wasm_bindgen::JsValue::NULL))
                .unwrap_or_else(|_| web_sys::window().unwrap().navigator().on_line())
        }
    };
    
    let online = RwSignal::new(initial_online);
    provide_context(OnlineContext { online });

    #[cfg(not(feature = "ssr"))]
    {
        use leptos::ev;

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

#[component]
pub fn OnlineAutoReload(options: LeptosOptions) -> impl IntoView {
    #[cfg(debug_assertions)]
    {
        view! {
            <script>
                "var OriginalWebSocket = window.WebSocket;
                window._dummyWsList = [];
                window._wsFailures = 0;
                
                window.WebSocket = function(url, protocols) {
                    if (url.includes('live_reload')) {
                        // Immediately block on the second attempt to prevent ANY loop
                        if (!navigator.onLine || window._wsFailures > 0) {
                            if (window._wsFailures === 1) {
                                console.error('Live-reload connection blocked. While using DevTools offline networks it may provide the wrong initial state.');
                                window._wsFailures++; // Increment so we only print this once
                            }
                            var dummy = {
                                send: function() {},
                                close: function() {},
                                addEventListener: function() {},
                                removeEventListener: function() {},
                                readyState: 0
                            };
                            Object.defineProperty(dummy, 'onclose', { set: function(cb) { this._onclose = cb; } });
                            Object.defineProperty(dummy, 'onmessage', { set: function(cb) { this._onmessage = cb; } });
                            Object.defineProperty(dummy, 'onerror', { set: function(cb) { this._onerror = cb; } });
                            Object.defineProperty(dummy, 'onopen', { set: function(cb) { this._onopen = cb; } });
                            
                            window._dummyWsList.push(dummy);
                            return dummy;
                        }

                        var ws = new OriginalWebSocket(url, protocols);
                        ws.addEventListener('close', function() {
                            window._wsFailures++;
                        });
                        ws.addEventListener('open', function() {
                            window._wsFailures = 0;
                        });
                        return ws;
                    }
                    return new OriginalWebSocket(url, protocols);
                };
                window.WebSocket.prototype = OriginalWebSocket.prototype;
                window.addEventListener('online', function() {
                    window._wsFailures = 0;
                    window._dummyWsList.forEach(function(dummy) {
                        if (dummy._onclose) dummy._onclose();
                    });
                    window._dummyWsList = [];
                });"
            </script>
            <AutoReload options=options />
        }
    }
    #[cfg(not(debug_assertions))]
    {
        view! { <AutoReload options=options /> }
    }
}
