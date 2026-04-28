#[cfg(not(feature = "ssr"))]
use leptos::prelude::*;


/// A trait for types that can be hydrated automatically.
///
/// This is ideal for data you **already have or can have on both sides** (e.g., cookies,
/// query params, or localStorage) to ensure a flicker-free initial render.
pub trait Hydratable: Send + Sync + 'static {
    /// Read from request details using isomorphic helpers like [`crate::get_cookie`] or [`crate::get_query_param`].
    ///
    /// - On SSR: read from HTTP request headers/URI. The result is serialised
    ///   into the HTML so the client never needs to re-compute it.
    /// - On client: used as a fallback when no injected value is found (CSR-only),
    ///   and re-run after hydration to synchronise with the client-side state.
    fn initial() -> Self
    where
        Self: Sized;

    /// Whether the library should re-run `initial()` on the client after hydration.
    ///
    /// Set this to `false` for states managed by **HttpOnly cookies**, as the client
    /// cannot read them and would incorrectly revert the state during synchronization.
    #[cfg(not(feature = "ssr"))]
    fn should_sync_on_client() -> bool
    where
        Self: Sized,
    {
        true
    }

    /// Optional hook called on the client after the signal is created and hydrated.
    #[cfg(not(feature = "ssr"))]
    fn on_hydrate(&self, _signal: RwSignal<Self>)
    where
        Self: Sized,
    {
    }
}
