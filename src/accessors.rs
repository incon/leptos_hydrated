use crate::core::HydratedSignal;
use leptos::prelude::*;

/// Access a signal provided by any `Hydrate*` component.
///
/// # Panics
/// Panics if no `HydratedSignal<T>` is found in context.
/// Use [`try_use_hydrated`] for a non-panicking alternative.
pub fn use_hydrated<T>() -> RwSignal<T>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<HydratedSignal<T>>().map(|s| s.0).expect(
        &format!(
            "HydratedSignal<{}> not found. Did you wrap this part of the tree in <HydrateState<{0}> />, <HydrateContext<{0}> />, <HydrateStateWith<{0}> />, or <HydrateContextWith<{0}> />?",
            std::any::type_name::<T>()
        )
    )
}

/// Non-panicking variant of [`use_hydrated`]. Returns `None` if no context is found.
pub fn try_use_hydrated<T>() -> Option<RwSignal<T>>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<HydratedSignal<T>>().map(|s| s.0)
}

/// Access the resource provided by any `Hydrate*` component.
///
/// # Panics
/// Panics if no resource is found in context.
/// Use [`try_use_hydrated_resource`] for a non-panicking alternative.
pub fn use_hydrated_resource<T>() -> LocalResource<Option<T>>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<LocalResource<Option<T>>>().unwrap_or_else(|| {
        panic!(
            "Hydrated LocalResource<{}> not found. Did you wrap this part of the tree in <HydrateState<{0}> />, <HydrateContext<{0}> />, <HydrateStateWith<{0}> />, or <HydrateContextWith<{0}> />?",
            std::any::type_name::<T>()
        )
    })
}

/// Non-panicking variant of [`use_hydrated_resource`]. Returns `None` if no context is found.
pub fn try_use_hydrated_resource<T>() -> Option<LocalResource<Option<T>>>
where
    T: Clone + Send + Sync + 'static,
{
    use_context::<LocalResource<Option<T>>>()
}
