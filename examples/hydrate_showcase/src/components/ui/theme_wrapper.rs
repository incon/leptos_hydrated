use leptos::prelude::*;
use leptos_hydrated::use_hydrated;
use crate::states::ProfileState;

/// Wraps the application to apply the global theme class synchronously
#[component]
pub fn ThemeWrapper(children: Children) -> impl IntoView {
    let state = use_hydrated::<ProfileState>();

    view! {
        <div class=move || format!("app-wrapper theme-{}", state.get().theme)>
            <main>{children()}</main>
        </div>
    }
}
