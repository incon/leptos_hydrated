use crate::core::create_hydrated_context;
use crate::traits::Hydratable;
use leptos::prelude::*;

/// Provides a hydrated signal for type `T` to its children.
///
/// This component initializes the hydrated state and provides both the `RwSignal<T>`
/// and the `LocalResource<Option<T>>` to all descendant components.
#[component]
pub fn HydratedContext<T>(
    /// Whether this context should be treated as a global provider.
    #[prop(default = false)]
    global: bool,
    /// A marker for the type parameter T.
    #[prop(optional)]
    _marker: std::marker::PhantomData<T>,
    /// The children of this context.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView
where
    T: Hydratable + Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + PartialEq + 'static,
{
    let (signal, resource) = create_hydrated_context::<T>();
    let _ = global;

    provide_context(signal);
    provide_context(resource);

    children.map(|c| c())
}
