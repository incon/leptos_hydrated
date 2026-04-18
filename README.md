# Leptos Hydrated

A lightweight library for **flicker-free interactive state hydration** in [Leptos 0.8](https://leptos.dev/) that works with or without JavaScript.

This library provides primitives to synchronize state from the server to the client seamlessly, ensuring that the initial render on the client matches the server-rendered HTML exactly, preventing "flashes" of default state before the client-side hydration completes.

## Features

- **Flicker-Free:** Initializes signals with server-provided state immediately during hydration.
- **Isomorphic:** Works naturally in both SSR and CSR contexts.
- **Global State:** Use the `Hydrate` component for global application state.
- **Scoped State:** Use the `HydrateContext` component for scoped feature state via context.
- **Warning-Free:** Uses `LocalResource` to avoid hydration mismatch warnings and optimize performance.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
leptos_hydrated = "0.4"
```

## Quick Start

### 1. Define your State

```rust
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ThemeState {
    pub theme: String,
}
```

### 2. Choose your hydration strategy

#### `Hydrate` (Global State)

Provides global state via context. It doesn't matter where you place it in the tree; the state is inherently global.

```rust
#[component]
pub fn App() -> impl IntoView {
    view! {
        // ssr_value and fetcher should ideally match on initial load 
        // to ensure zero visual flickering.
        <Hydrate
            ssr_value=move || ThemeState { theme: "dark".into() }
            fetcher=|| async { Ok(ThemeState { theme: "dark".into() }) }
        />
        <MainContent />
    }
}

#[component]
fn MainContent() -> impl IntoView {
    let state = use_hydrated::<ThemeState>();
    view! {
        <p>"Theme: " {move || state.get().theme}</p>
    }
}
```

#### `HydrateContext` (Scoped State)

Provides scoped feature state to all descendants via the standard Leptos context API. By matching `ssr_value` with the initial `fetcher` state, you ensure zero visual flickering during the hydration transition.

```rust
#[component]
fn ProfileSection() -> impl IntoView {
    view! {
        // By using the same source (e.g., a cookie) for both,
        // you guarantee a perfectly smooth hydration hand-off.
        <HydrateContext
            ssr_value=|| ThemeState { theme: "dark".into() }
            fetcher=|| async { Ok(ThemeState { theme: "dark".into() }) }
        >
            <ThemedComponent />
        </HydrateContext>
    }
}

#[component]
fn ThemedComponent() -> impl IntoView {
    let state = use_hydrated::<ThemeState>();
    view! { <div class=move || state.get().theme> "Flicker-free theme!" </div> }
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
