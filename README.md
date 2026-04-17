# Leptos Hydrated

A lightweight library for **flicker-free interactive state hydration** in [Leptos 0.8](https://leptos.dev/).

This library provides primitives to synchronize state from the server to the client seamlessly, ensuring that the initial render on the client matches the server-rendered HTML exactly, preventing "flashes" of default state before the client-side hydration completes.

## Features

- **Flicker-Free:** Initializes signals with server-provided state immediately during hydration.
- **Isomorphic:** Works naturally in both SSR and CSR contexts.
- **Context Integration:** Provide global hydrated state via `HydrateContext`.
- **Scoped Hydration:** Use the `Hydrate` component for localized hydrated state.
- **Resource Support:** Automatically manages a background `Resource` to keep data in sync.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
leptos_hydrated = "0.2.0"
```

## Quick Start

### 1. Define your State

```rust
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    pub theme: String,
    pub user_name: Option<String>,
}
```

### 2. Wrap your App in `HydrateContext`

Use `ssr_value` to provide the immediate synchronous state (e.g., from cookies) and `fetcher` for the full asynchronous data load.

```rust
#[component]
pub fn App() -> impl IntoView {
    view! {
        <HydrateContext
            ssr_value=move || read_initial_state_from_cookies()
            fetcher=|| fetch_full_app_state()
        >
            <MainContent />
        </HydrateContext>
    }
}
```

### 3. Use the state in components

```rust
#[component]
fn MainContent() -> impl IntoView {
    let state = use_hydrated::<AppState>();

    view! {
        <p>"Welcome, " {move || state.get().user_name.unwrap_or_else(|| "Guest".to_string())}</p>
    }
}
```

## Best Practices: Specialized Contexts

For larger applications, it is a best practice to wrap `HydrateContext` in specialized components. This keeps your `App` component clean and allows you to place contexts exactly where they are needed.

```rust
#[component]
fn ProfileContext(children: Children) -> impl IntoView {
    view! {
        <HydrateContext 
            ssr_value=read_profile_state 
            fetcher=fetch_profile_state
        >
            {children()}
        </HydrateContext>
    }
}

view! {
    <ProfileContext>
        <Router>
            <Routes>
                <Route path=StaticSegment("") view=HomePage/>
            </Routes>
        </Router>
    </ProfileContext>
}
```

## Example Project

A full demonstration is available in the `examples/hydrate_showcase` directory. It features:
- Dark/Light mode synchronization via cookies.
- Authentication state persistence.
- URL Parameter synchronization with hydrated state.

To run the example:
```bash
cd examples/hydrate_showcase
cargo leptos watch
```
