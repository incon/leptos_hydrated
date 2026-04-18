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
leptos_hydrated = "0.5"
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
        // Read from cookie/URL synchronously
        ThemeState { theme: "dark".into() }
    }

    async fn fetch() -> Result<Self, ServerFnError> {
        // Use state already in the browser or refresh from API asynchronously
        Ok(ThemeState { theme: "light".into() })
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

## Why use this instead of a standard `Resource`?

Standard Leptos `Resource`s are fantastic for data that lives on the server and needs to be serialized to the client. However, they can cause "flickers" or require `Suspense` masks for data you **already have** on both sides (like a cookie).

`leptos_hydrated` allows you to:
1.  **Render immediately** on the server using a synchronous value.
2.  **Hydrate immediately** on the client with that same value (no flicker!).
3.  **Refresh in the background** once the WASM is ready to get the latest data.

## Documentation

Full API documentation is available at [docs.rs/leptos_hydrated](https://docs.rs/leptos_hydrated).
