use crate::core::use_hydrated_context;
use crate::traits::Hydratable;
use leptos::prelude::*;

/// Provides hydrated state using the [`Hydratable`] trait.
///
/// This component automatically manages a hydrated signal and provides it
/// via context to its descendants.
///
/// If `children` are provided, it scopes the state to those children.
/// If `global=true` is set, or if it has no children, it acts as a global provider
/// for its reactive scope.
///
/// Use `hydrated_signal(T::initial())` in any descendant to access the signal.
#[component]
pub fn HydratedContext<T>(
    #[prop(optional)] children: Option<ChildrenFn>,
    #[prop(optional)] global: bool,
    #[prop(optional)] _marker: std::marker::PhantomData<T>,
) -> impl IntoView
where
    T: Hydratable
        + Clone
        + Send
        + Sync
        + serde::Serialize
        + serde::de::DeserializeOwned
        + PartialEq
        + 'static,
{
    let _ = global;
    let state = use_hydrated_context::<T>();

    let children_view = if let Some(children_fn) = children {
        // Scoped to children: create a child owner so we don't leak context
        let owner = leptos::prelude::Owner::current()
            .expect("no current reactive Owner found")
            .child();
        let view = owner.with(|| {
            provide_context(state);
            provide_context(state.resource);
            children_fn()
        });
        Some(leptos::tachys::reactive_graph::OwnedView::new_with_owner(
            view, owner,
        ))
    } else {
        // Global/Sibling provider: provide directly to current owner
        provide_context(state);
        provide_context(state.resource);
        None
    };

    view! { {children_view} }
}
