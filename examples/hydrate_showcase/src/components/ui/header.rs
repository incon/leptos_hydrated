use leptos::prelude::*;
use leptos_hydrated::use_hydrated;
use crate::states::ProfileState;

#[component]
pub fn Header() -> impl IntoView {
    let profile_state = use_hydrated::<ProfileState>();

    view! {
        <header class="top-nav">
            <h1>"Hydrate Showcase"</h1>
            <div class="controls">
                <button class="btn btn-secondary" on:click=move |_| ProfileState::toggle_theme(profile_state)>
                    "Switch to "
                    {move || if profile_state.get().theme == "dark" { "Light" } else { "Dark" }}
                </button>
                <button class="btn btn-primary" on:click=move |_| ProfileState::toggle_login(profile_state)>
                    {move || {
                        if profile_state.get().is_authenticated { "Log Out" } else { "Log In" }
                    }}
                </button>
            </div>
        </header>
    }
}
