use crate::states::ThemeState;
use leptos::prelude::*;
use leptos_hydrated::*;

/// Wraps the application to apply the global theme class synchronously
#[component]
pub fn ThemeWrapper(children: Children) -> impl IntoView {
    let state = use_hydrated_context::<ThemeState>();
    provide_context(state);

    view! {
        <div class=move || format!("app-wrapper theme-{}", state.get().0)>
            <main>{children()}</main>
        </div>
    }
}
