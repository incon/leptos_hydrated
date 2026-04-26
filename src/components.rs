use crate::core::{HydratedSignal, use_hydrate_signal};
use crate::traits::Hydratable;
use leptos::prelude::*;

#[cfg(any(feature = "ssr", target_arch = "wasm32"))]
use crate::core::type_hydration_id;

#[cfg(feature = "ssr")]
use crate::core::serialize_for_injection;

/// Provides hydrated state using the [`Hydratable`] trait.
///
/// Injects the server value into an inline `<script>` tag so the client can
/// read it immediately — no flicker. The script tag is always rendered on
/// both SSR and client to keep the DOM structure identical for hydration.
///
/// If `children` are provided, it scopes the state to those children.
/// If `global=true` is set, or if it has no children, it acts as a global provider
/// for its reactive scope.
///
/// Use `use_hydrated::<T>()` in any descendant to access the signal.
#[component]
pub fn HydrateContext<T>(
    #[prop(optional)] children: Option<ChildrenFn>,
    #[prop(optional)] global: bool,
    #[prop(optional)] _marker: std::marker::PhantomData<T>,
) -> impl IntoView
where
    T: Hydratable + PartialEq + 'static,
{
    let _ = global;
    let (signal, resource) = use_hydrate_signal::<T>();

    let children_view = if let Some(children_fn) = children {
        // Scoped to children: create a child owner so we don't leak context
        let owner = leptos::prelude::Owner::current()
            .expect("no current reactive Owner found")
            .child();
        let view = owner.with(|| {
            provide_context(HydratedSignal(signal));
            provide_context(resource);
            children_fn()
        });
        Some(leptos::tachys::reactive_graph::OwnedView::new_with_owner(
            view, owner,
        ))
    } else {
        // Global/Sibling provider: provide directly to current owner
        provide_context(HydratedSignal(signal));
        provide_context(resource);
        None
    };

    view! {
        {children_view}
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
