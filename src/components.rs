use crate::core::{HydratedSignal, use_hydrate_signal};
use crate::traits::Hydratable;
use leptos::prelude::*;

#[cfg(any(feature = "ssr", target_arch = "wasm32"))]
use crate::core::type_hydration_id;

#[cfg(feature = "ssr")]
use crate::core::serialize_for_injection;

/// Provides global hydrated state using the [`Hydratable`] trait.
///
/// Injects the server value into an inline `<script>` tag so the client can
/// read it immediately — no flicker. The script tag is always rendered on
/// both SSR and client to keep the DOM structure identical for hydration.
/// Use `use_hydrated::<T>()` in any descendant to access the signal.
#[component]
pub fn HydrateState<T>(#[prop(optional)] _marker: std::marker::PhantomData<T>) -> impl IntoView
where
    T: Hydratable + PartialEq,
{
    let (signal, resource) = use_hydrate_signal::<T>();
    provide_context(HydratedSignal(signal));
    provide_context(resource);

    #[cfg(any(feature = "ssr", target_arch = "wasm32"))]
    {
        let id = type_hydration_id::<T>();
        let script_id = format!("__lh_{}", id);
        view! {
            <script type="application/json" id={script_id}
                inner_html={
                    #[cfg(feature = "ssr")]
                    { serialize_for_injection(&T::initial()) }
                    #[cfg(not(feature = "ssr"))]
                    { "" }
                }
            />
        }
    }
    #[cfg(all(not(feature = "ssr"), not(target_arch = "wasm32")))]
    {
        view! {}
    }
}

/// Provides scoped hydrated state using the [`Hydratable`] trait.
///
/// Injects the server value and renders children inside the same component.
/// Use `use_hydrated::<T>()` in any child to access the signal.
#[component]
pub fn HydrateContext<T>(
    children: Children,
    #[prop(optional)] _marker: std::marker::PhantomData<T>,
) -> impl IntoView
where
    T: Hydratable + PartialEq,
{
    let (signal, resource) = use_hydrate_signal::<T>();
    provide_context(HydratedSignal(signal));
    provide_context(resource);
    view! {
        {children()}
        {
            #[cfg(any(feature = "ssr", target_arch = "wasm32"))]
            {
                let id = type_hydration_id::<T>();
                let script_id = format!("__lh_{}", id);
                view! {
                    <script type="application/json" id={script_id}
                        inner_html={
                            #[cfg(feature = "ssr")]
                            { serialize_for_injection(&T::initial()) }
                            #[cfg(not(feature = "ssr"))]
                            { "" }
                        }
                    />
                }
            }
            #[cfg(all(not(feature = "ssr"), not(target_arch = "wasm32")))]
            {
                view! { }
            }
        }
    }
}
