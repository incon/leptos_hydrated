# Leptos Hydrated

A lightweight library for **flicker-free interactive state hydration** in [Leptos 0.8](https://leptos.dev/) that works with or without JavaScript.

This library provides primitives to synchronize state from the server to the client seamlessly, ensuring that the initial render on the client matches the server-rendered HTML exactly, preventing "flashes" of default state before the client-side hydration completes.

## Features

- **Flicker-Free:** Initializes signals with server-provided state immediately during hydration.
- **Isomorphic:** Works naturally in both SSR and CSR contexts.
- **Global State:** Use the `Hydrate` component for application-wide state via a render prop.
- **Scoped State:** Use the `HydrateContext` component for localized feature state via context.
- **Resource Support:** Automatically manages a background `Resource` to keep data in sync.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
leptos_hydrated = "0.2.1"
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

### 2. Choose your hydration strategy

#### `Hydrate` (Global State)

Provides global state to its children via a render prop. It doesn't matter where you place it in the tree; the state remains global to the provided closure.

```rust
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Hydrate
            ssr_value=move || read_initial_state_from_cookies()
            fetcher=|| fetch_full_app_state()
            children=move |state| {
                view! { <MainContent state /> }
            }
        />
    }
}
```

#### `HydrateContext` (Scoped State)

Provides scoped feature state to all descendants via the standard Leptos context API. Best for specialized features that need deep access without prop drilling.

```rust
#[component]
fn FeatureSection() -> impl IntoView {
    view! {
        <HydrateContext
            ssr_value=|| "Guest".to_string()
            fetcher=|| async { Ok("Leptos User".to_string()) }
        >
            <SubComponent />
        </HydrateContext>
    }
}

#[component]
fn SubComponent() -> impl IntoView {
    let name = use_hydrated::<String>();
    view! { <p>"User: " {move || name.get()}</p> }
}
```

## Best Practices: Specialized Contexts

For larger applications, it is a best practice to wrap `HydrateContext` in specialized components for specific feature scopes.

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
