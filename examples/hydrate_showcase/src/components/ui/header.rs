use leptos::prelude::*;
use leptos::form::ActionForm;
use leptos_hydrated::use_hydrated;
use crate::states::{ProfileState, ToggleThemeServer, ToggleLoginServer};

#[component]
pub fn Header() -> impl IntoView {
    let profile_state = use_hydrated::<ProfileState>();
    let toggle_theme = ServerAction::<ToggleThemeServer>::new();
    let toggle_login = ServerAction::<ToggleLoginServer>::new();

    view! {
        <header class="top-nav">
            <h1>"Hydrate Showcase"</h1>
            <div class="controls">
                <ActionForm action=toggle_theme>
                    <button type="submit" class="btn btn-secondary">
                        "Switch to "
                        {move || if profile_state.get().theme == "dark" { "Light" } else { "Dark" }}
                    </button>
                </ActionForm>
                <ActionForm action=toggle_login>
                    <button type="submit" class="btn btn-primary">
                        {move || {
                            if profile_state.get().is_authenticated { "Log Out" } else { "Log In" }
                        }}
                    </button>
                </ActionForm>
            </div>
        </header>
    }
}
