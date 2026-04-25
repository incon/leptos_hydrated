# Leptos Hydrated

A lightweight library for **flicker-free interactive state hydration** in [Leptos 0.8](https://leptos.dev/). It is designed for data you **already have or can have on both sides** (isomorphic data), such as cookies, URL parameters, or locally cached state.

## Core Philosophy

`leptos_hydrated` is ideal for bootstrapping state that is available on both the server and the client without waiting for an API call. By initializing signals immediately with server-provided state and synchronizing them once the browser is active, you eliminate the "loading flicker" common in SSR applications.

## Features

- **Flicker-Free:** Initializes signals with server-provided state immediately during hydration.
- **Isomorphic-First:** Perfect for data available on both sides (cookies, URL params, window state).
- **Browser-Ready:** Leverage state already in the browser to render the first frame without waiting for async resources.
- **Trait-Based:** Use the `Hydratable` trait to define state and refresh logic in one place.
- **Global & Scoped:** Support for both global application state and scoped feature state.
- **Zero Mismatch:** Designed to avoid hydration warnings by matching server and client initial renders exactly.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
leptos_hydrated = "0.8"
```

## How it Works

1. **Server-Side Render (SSR):** `initial()` is called on the server. The result is serialized into the HTML shell.
2. **Hydration:** The client reads the serialized state from the HTML and initializes the signal immediately — **zero flicker**.
3. **Synchronization:** Once the WASM is active, `initial()` is re-run on the client to synchronize with the current browser state (e.g., reading a JS-accessible cookie).
4. **Lifecycle Hooks:** Use `on_hydrate` to set up browser-only event listeners (e.g., network status, window resize).

## Quick Start

### 1. Define your State with `Hydratable`

To use `leptos_hydrated`, you implement the `Hydratable` trait. This encapsulates your synchronous "seed" logic (e.g., cookies) and your asynchronous "refresh" logic (e.g., API calls).

```rust
use leptos::prelude::*;
use leptos_hydrated::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct ThemeState {
    pub theme: String,
}

impl Hydratable for ThemeState {
    fn initial() -> Self {
        // Use isomorphic helpers to read from cookies/query params on both sides.
        let theme = get_cookie("theme").unwrap_or_else(|| "dark".into());
        ThemeState { theme }
    }

    #[cfg(not(feature = "ssr"))]
    fn on_hydrate(&self, state: RwSignal<Self>) {
        // Optional: Do something in the browser after hydration
    }

    #[cfg(not(feature = "ssr"))]
    fn should_sync_on_client() -> bool {
        // Optional: Opt-out of client-side synchronization
        true
    }
}
```

### 2. Choose your hydration strategy

#### `HydrateState` (Global State)

Provides global state via context. Place it anywhere in your view tree.

```rust
#[component]
pub fn App() -> impl IntoView {
    view! {
        // 1. Provide state anywhere in the tree
        <HydrateState<ThemeState> />
        
        <MainContent />
    }
}

#[component]
fn MainContent() -> impl IntoView {
    // 2. Consume it anywhere in the tree
    let state = use_hydrated::<ThemeState>();
    view! {
        <p>"Theme: " {move || state.get().theme}</p>
    }
}
```

#### `HydrateContext` (Scoped State)

Provides scoped state to a specific branch of the component tree.

```rust
#[component]
fn ProfileSection() -> impl IntoView {
    view! {
        <HydrateContext<UserState>>
            <ProfileInfo />
        </HydrateContext<UserState>>
    }
}
```

### Environment Macros

`leptos_hydrated` provides macros to simplify environment-gated code:

- **`isomorphic! { state => ..., hydrate => ... }`**: A concise way to provide server state and browser hydration.
- **`get_injected_state<T>()`**: Allows the client to inspect the value originally sent by the server for state merging.
- **`server_only! { ... }`**: Executes code only on the server. Returns `()` in the browser.
- **`client_only! { ... }`**: Executes code only in the browser. Returns `()` on the server.
- **`is_server()`**: Returns `true` if running on the server.
- **`is_client()`**: Returns `true` if running in the browser.

#### Example: Merging Server & Local State

```rust
impl Hydratable for TodoState {
    fn initial() -> Self {
        isomorphic! {
            state => Self::default(),
            hydrate => {
                // Get state injected by the server (this block only runs in the browser)
                let from_server = get_injected_state::<Self>();

                // Prioritize local state (localStorage) over server state
                read_from_local_storage()
                    .or(from_server)
                    .unwrap_or_default()
            }
        }
    }
}
```

### Isomorphic Helpers

These helpers read and write state consistently on both server and client.

- **`get_cookie(name)`**: Reads a cookie by name. 
  - *SSR:* Reads from `http::request::Parts`.
  - *Client:* Reads from `document.cookie`.
- **`set_cookie(name, value, options)`**: Sets a cookie. 
  - *SSR:* Inserts a `SET-COOKIE` header into the response.
  - *Client:* Updates `document.cookie`.
- **`get_query_param(name)`**: Reads a URL query parameter.
  - *SSR:* Reads from current URI or `Referer` fallback.
  - *Client:* Reads from `window.location.search`.

## PWA & "Born Offline" Support

Progressive Web Apps often load from an "offline shell" (an empty HTML file cached by a Service Worker). In this scenario, the app is **not** hydrated from SSR content but is instead "Born Offline" in CSR mode.

`leptos_hydrated` handles this by allowing you to propagate the mounting mode from your `lib.rs` into your component tree.

### 1. Detect Mounting Mode (lib.rs)

In your PWA entry point, check if the DOM already contains UI. If not, you are running from the offline shell.

```rust
#[wasm_bindgen]
pub fn hydrate() {
    let body = document().body().unwrap();
    // If there is no UI, we are in the offline shell
    let was_hydrated = body.query_selector(":not(script)").unwrap().is_some();

    if !was_hydrated {
        leptos::mount::mount_to_body(move || {
            view! { <Pwa was_hydrated=false><App /></Pwa> }
        });
    } else {
        leptos::mount::hydrate_body(move || {
            view! { <Pwa was_hydrated=true><App /></Pwa> }
        });
    }
}
```

### 2. Provide Context via a Wrapper

Create a simple wrapper to provide the hydration status to your states.

```rust
#[derive(Copy, Clone, Debug)]
pub struct PwaInit { pub was_hydrated: bool }

#[component]
pub fn Pwa(children: Children, was_hydrated: bool) -> impl IntoView {
    provide_context(PwaInit { was_hydrated });
    children()
}
```

### 3. Consume in your State

Use the context in `initial()` to decide how to seed your state. This is perfect for restoring from `localStorage` or correctly setting initial connectivity status.

```rust
impl Hydratable for OnlineState {
    fn initial() -> Self {
        // Detect if we were hydrated from SSR or started from an offline shell
        let was_hydrated = use_context::<PwaInit>()
            .map(|c| c.was_hydrated)
            .unwrap_or(true);

        isomorphic! {
            state => Self { online: true },
            hydrate => Self { online: was_hydrated }
        }
    }

    #[cfg(not(feature = "ssr"))]
    fn on_hydrate(&self, state: RwSignal<Self>) {
        // Set up browser-only event listeners to keep state in sync
        use leptos::ev;
        let _ = use_event_listener(web_sys::window(), ev::online, move |_| {
            state.update(|s| s.online = true);
        });
        let _ = use_event_listener(web_sys::window(), ev::offline, move |_| {
            state.update(|s| s.online = false);
        });
    }
}
```

## Server-Side Setup

In order for the isomorphic helpers to access request data on the server, you **must** use `.leptos_routes_with_context` in your Axum server setup and call `provide_hydration_context()`. This initializes the request-scoped store for cookies and query parameters.

```rust
// src/main.rs (Server)
let app: Router = Router::new()
    .leptos_routes_with_context(
        &leptos_options,
        routes,
        || {
            // This initializes the hydration store from the current request
            leptos_hydrated::provide_hydration_context();
        },
        move || shell(),
    )
    .with_state(leptos_options);
```

## Why use this instead of a standard `Resource`?

Standard Leptos `Resource`s are fantastic for data that lives on the server and needs to be serialized to the client. However, they can cause "flickers" or require `Suspense` masks for data you **already have** on both sides (like a cookie).

`leptos_hydrated` allows you to:
1.  **Render immediately** on the server using a synchronous value.
2.  **Hydrate immediately** on the client with that same value (no flicker!).
3.  **Refresh in the background** once the WASM is ready to get the latest data.

### Secure Hydration (HTTP-only Cookies)

When using sensitive data like authentication tokens in HTTP-only cookies, the client JavaScript cannot read the cookie to initialize state. `leptos_hydrated` solves this by allowing the server to read the cookie, fetch the corresponding user data, and inject *only the result* into the HTML.

The client hydrates the user data synchronously, while the secret token remains hidden from JavaScript.

## Documentation

Full API documentation is available at [docs.rs/leptos_hydrated](https://docs.rs/leptos_hydrated).
