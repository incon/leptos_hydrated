use crate::core::HydrateSignal;
use leptos::prelude::*;

/// A helper for accessing hydrated state via context.
pub struct Hydrated<T>(std::marker::PhantomData<T>);

impl<T> Hydrated<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Access a signal provided by any `Hydrate*` component.
    ///
    /// # Panics
    /// Panics if no `HydrateSignal<T>` is found in context.
    pub fn get() -> RwSignal<T> {
        Self::try_get().expect(&format!(
            "HydrateSignal<{}> not found. Did you wrap this part of the tree in <HydratedContext<{0}> />?",
            std::any::type_name::<T>()
        ))
    }

    /// Non-panicking variant of [`get`]. Returns `None` if no context is found.
    pub fn try_get() -> Option<RwSignal<T>> {
        use_context::<HydrateSignal<T>>().map(|s| s.signal)
    }

    /// Access the resource provided by any `Hydrate*` component.
    ///
    /// # Panics
    /// Panics if no resource is found in context.
    pub fn resource() -> LocalResource<Option<T>> {
        Self::try_resource().expect(&format!(
            "Hydrated LocalResource<{}> not found. Did you wrap this part of the tree in <HydratedContext<{0}> />?",
            std::any::type_name::<T>()
        ))
    }

    /// Non-panicking variant of [`resource`]. Returns `None` if no context is found.
    pub fn try_resource() -> Option<LocalResource<Option<T>>> {
        use_context::<HydrateSignal<T>>()
            .map(|s| s.resource)
            .or_else(|| use_context::<LocalResource<Option<T>>>())
    }
}
