# Leptos Hydrated

A library for **flicker-free interactive state hydration** in [Leptos 0.8](https://leptos.dev/).

`leptos_hydrated` is ideal for bootstrapping state that you **already have or can have on both sides** (isomorphic data), such as cookies, URL parameters, or locally cached state. By initializing signals immediately with server-provided state and synchronizing them once the browser is active, you eliminate the "loading flicker" common in SSR applications.

## How it Works

1.  **Server-Side Render (SSR):** `initial()` is called on the server. The result is serialized into the HTML shell.
2.  **Hydration:** The client reads the serialized state from the HTML and initializes the signal immediately — **zero flicker**.
3.  **Synchronization:** Once the WASM is active, `initial()` is re-run on the client to synchronize with the current browser state (e.g., reading a JS-accessible cookie).
4.  **Lifecycle Hooks:** Use `on_hydrate` to execute any client-side code immediately after hydration (e.g., event listeners, storage synchronization).

## Hydration Scopes

`leptos_hydrated` offers three levels of state scope, ordered by increasing granularity:

### 1. Local

Use `hydrated_signal` directly in a component. This creates a hydrated signal that is unique to this component instance and is **not** shared via context.

```rust
#[component]
fn MyComponent() -> impl IntoView {
    // This state is unique to this instance of MyComponent
    let state = hydrated_signal(MyState::initial());
    // ...
}
```

### 2. Scoped

Wrap a section of your component tree with `<HydratedContext<T>>`. This provides the hydrated state to all descendants in that subtree.

```rust
#[component]
fn Feature() -> impl IntoView {
    view! {
        <HydratedContext<MyState>>
            // All descendants can access the same MyState
            <Descendant />
        </HydratedContext<MyState>>
    }
}
```

### 3. Global

Use `<HydratedContext<T> global=true />` (typically in your app shell). This provides the state globally across your entire application.

```rust
#[component]
fn App() -> impl IntoView {
    view! {
        <HydratedContext<MyState> global=true />
        // MyState is now available everywhere in the app
        <MainContent />
    }
}
```

## Quick Start

### 1. Define your State with `Hydratable`

Implement the [`Hydratable`] trait to define how your state is initialized and synchronized.

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
        // Use isomorphic helpers to read from cookies on both sides.
        let theme = get_cookie("theme").unwrap_or_else(|| "dark".into());
        ThemeState { theme }
    }

    #[cfg(not(feature = "ssr"))]
    fn on_hydrate(&self, state: RwSignal<Self>) {
        // Optional: Execute code in the browser after hydration
    }
}
```

### 2. Accessing Hydrated State

You can use `hydrated_signal(T::initial())` to access state. It will automatically check if a provider (from `HydratedContext`) exists in the context; if so, it uses the shared signal, otherwise it creates a local one.

```rust
#[component]
fn MainContent() -> impl IntoView {
    let state = hydrated_signal(ThemeState::initial());
    view! {
        <p>"Theme: " {move || state.get().theme}</p>
    }
}
```

## Server-Side Setup

### Middleware

You **must** add the `.hydrated()` middleware to your Axum router. This middleware handles collecting the state during rendering and injecting it into the HTML. It also provides the necessary request context for isomorphic helpers (like `get_cookie`).

```rust
// src/main.rs (Server)
use leptos_hydrated::HydratedRouterExt;

let app = Router::new()
    .leptos_routes(&leptos_options, routes, {
        let leptos_options = leptos_options.clone();
        move || shell(leptos_options)
    })
    .fallback(leptos_axum::file_and_error_handler)
    .hydrated() // <--- Add this before .with_state()
    .with_state(leptos_options);
```

## Isomorphic Helpers

These helpers read and write state consistently on both server and client.

- **`get_cookie(name)`**: Reads a cookie by name. 
- **`set_cookie(name, value, options)`**: Sets a cookie.
- **`get_query_param(name)`**: Reads a URL query parameter.

## Environment Utilities

- **`isomorphic! { state => ..., hydrate => ... }`**: Run different logic for server seed vs client hydration.
- **`use_hydrated_context<T>()`**: Accesses the hydrated state from context (returns `Option<HydrateSignal<T>>`).
- **`inject_state(&value)`**: Manually inject a state from the server (SSR only).
- **`use_injected_state<T>()`**: Reads the next available injected state from the server (client-side only).

### Example: Manual Injection with `isomorphic!`

You can use `inject_state()` on the server and `use_injected_state<T>()` in the browser to handle custom state hydration within an `isomorphic!` block.

```rust
let my_value = isomorphic! {
    state => {
        let value = MyState { count: 42 };
        inject_state(&value); // Push to injection stream
        value
    },
    hydrate => {
        use_injected_state::<MyState>().unwrap_or_else(|| MyState { count: 0 })
    }
};
```

