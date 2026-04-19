# Leptos Hydrated

A lightweight library for **flicker-free interactive state hydration** in [Leptos 0.8](https://leptos.dev/) that works with or without JavaScript.

## Features

- **Flicker-Free:** Initializes signals with server-provided state immediately during hydration.
- **Browser-First:** Leverage state already in the browser (cookies, URL params) to render the first frame without waiting for API calls.
- **Isomorphic:** Works naturally in both SSR and CSR contexts.
- **Trait-Based:** Use the `Hydratable` trait to define state and refresh logic in one place.
- **Global & Scoped:** Support for both global application state and scoped feature state.
- **Zero Mismatch:** Designed to avoid hydration warnings by matching server and client initial renders exactly.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
leptos_hydrated = "0.6"
```

## Quick Start

### 1. Define your State with `Hydratable`

The most robust way to use `leptos_hydrated` is by implementing the `Hydratable` trait. This encapsulates your synchronous "seed" logic (e.g., cookies) and your asynchronous "refresh" logic (e.g., API calls).

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

    fn fetch() -> impl Future<Output = Option<Result<Self, ServerFnError>>> + Send + 'static {
        // Return None if you only want to use the injected value.
        // Return Some(Result) if you want to refresh the state in the background.
        async { None }
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

### 3. Manual Hydration (Advanced)

If you don't want to use the trait, you can use the base components directly:

```rust
view! {
    // Global
    <HydrateStateWith
        ssr_value=|| ThemeState { theme: "dark".into() }
        fetcher=|| async { Ok(ThemeState { theme: "light".into() }) }
    />
    
    // Scoped
    <HydrateContextWith
        ssr_value=|| ThemeState { theme: "dark".into() }
        fetcher=|| async { Ok(ThemeState { theme: "light".into() }) }
    >
        <ProfileInfo />
    </HydrateContextWith>
}
```

### Isomorphic Helpers

`leptos_hydrated` provides several helpers to read and write state consistently on both server and client, which is particularly useful inside `Hydratable::initial()`.

- **`get_cookie(name)`**: Reads a cookie by name. 
  - *SSR:* Reads from `http::request::Parts`.
  - *Client:* Reads from `document.cookie`.
- **`set_cookie(name, value, options)`**: Sets a cookie. 
  - *SSR:* Uses `leptos_axum::ResponseOptions` to insert a `SET-COOKIE` header.
  - *Client:* Updates `document.cookie`.
- **`get_query_param(name)`**: Reads a URL query parameter. 
  - *SSR:* Reads from the request URI.
  - *Client:* Reads from `window.location.search`.
- **`get_referer_query_param(name)`**: Reads a query parameter from the `Referer` header. 
  - *Note:* Essential for server functions where the current request URI is the endpoint, but you need the original page's context.
- **`get_header(name)`**: Reads an arbitrary HTTP header by name.
  - *SSR:* Reads from `http::request::Parts`.
  - *Client:* Returns `None`.
- **`set_header(name, value)`**: Sets an arbitrary HTTP header.
  - *SSR:* Inserts into `leptos_axum::ResponseOptions`.
  - *Client:* No-op.

## Server-Side Setup

In order for the isomorphic helpers to access request data on the server, you **must** use `.leptos_routes_with_context` in your Axum server setup. This provides the `http::request::Parts` and `leptos_axum::ResponseOptions` to the Leptos context.

```rust
// src/main.rs (Server)
let app = Router::new()
    .leptos_routes_with_context(
        &leptos_options,
        routes,
        || {}, // Provide additional context here if needed
        {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        },
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
