# Leptos Hydrated

A library for flicker-free interactive state hydration in [Leptos 0.8](https://leptos.dev/).

`leptos_hydrated` is ideal for bootstrapping state that you already have or can have on both sides (isomorphic data), such as cookies or URL parameters. By initializing signals immediately with server-provided state and synchronizing them once the browser is active, you eliminate the "loading flicker" common in SSR applications.

## How it Works

1.  **Server-Side Render (SSR):** `initial()` is called on the server. The result is serialized into a deterministic injection stream in the HTML shell.
2.  **Hydration:** The client reads the serialized state from the stream in the same order and initializes the signal immediately: **zero flicker**.
3.  **Synchronization:** Once the WASM is active, `initial()` is re-run on the client to synchronize with the current browser state (e.g., reading a JS-accessible cookie).
4.  **Lifecycle Hooks:** Use `on_hydrate` to execute any client-side code immediately after hydration (e.g., event listeners).

## Hydration Accessors

`leptos_hydrated` mirrors standard Leptos signal patterns to make state management intuitive.

### 1. Local (Independent)

Use `hydrated_signal` to create a new, independent hydrated signal. This works exactly like `RwSignal::new(T)`, but it is hydration-aware.

```rust
#[component]
fn MyComponent() -> impl IntoView {
    // This state is unique to this component instance
    let state = hydrated_signal(MyState::initial());
    
    view! {
        <p>"Count: " {move || state.get().count}</p>
    }
}
```

### 2. Scoped (Shared)

Wrap a section of your component tree with `<HydratedContext<T>>` to share a hydrated signal. Use `use_hydrated_context<T>()` in descendants to access it.

```rust
#[component]
fn Feature() -> impl IntoView {
    view! {
        <HydratedContext<MyState>>
            <Descendant />
        </HydratedContext<MyState>>
    }
}

#[component]
fn Descendant() -> impl IntoView {
    // Access the shared signal from context
    let state = use_hydrated_context::<MyState>();
    
    view! {
        <p>{move || state.get().name}</p>
    }
}
```

### 3. Global (Shared)

Use `<HydratedContext<T> global=true />` (typically in your app shell) to provide state globally across your entire application.

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

Implement the `Hydratable` trait to define how your state is initialized and synchronized.

```rust
use leptos::prelude::*;
use leptos_hydrated::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct ThemeState(pub String);

impl Hydratable for ThemeState {
    fn initial() -> Self {
        // Use isomorphic helpers to read from cookies on both sides.
        let theme = get_cookie("theme").unwrap_or_else(|| "dark".into());
        ThemeState(theme)
    }

    #[cfg(not(feature = "ssr"))]
    fn on_hydrate(&self, state: RwSignal<Self>) {
        // Optional: Execute code in the browser immediately after hydration
        leptos::logging::log!("Theme hydrated: {}", self.0);
    }
}
```

## Bundle Size Optimization

`leptos_hydrated` is designed to keep your client-side WASM bundles as lean as possible. By default, Leptos server functions rely on `serde_json` for communication, which can add **~150-200 KB** to your WASM binary.

### `#[hydrated_server]`

Use the `#[hydrated_server]` macro instead of standard `#[server]` to eliminate `serde_json` from your frontend. It uses the browser's native `JSON` primitives for serialization/deserialization.

```rust
#[hydrated_server]
pub async fn update_user(name: String) -> Result<User, ServerFnError> {
    // This function uses native Browser JSON on the client
    // and serde_json on the server automatically.
    Ok(User { name })
}
```

This optimization is fully compatible with `ActionForm` and standard `ServerAction` patterns.

## Server-Side Setup

### 1. Middleware

You **must** add the `.hydrated()` middleware to your Axum router.

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

- **`get_cookie(name)`**: Reads a cookie by name. 
- **`set_cookie(name, value, options)`**: Sets a cookie.
- **`get_query_param(name)`**: Reads a URL query parameter.

## Utilities

- **`isomorphic! { state => ..., hydrate => ... }`**: Branch logic for server-seed vs client-hydration.
- **`use_hydrated_context<T>()`**: Accesses state from context (returns `Option<RwSignal<T>>`).
