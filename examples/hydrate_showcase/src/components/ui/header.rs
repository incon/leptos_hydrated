use crate::states::{ProfileState, ThemeState, ToggleLoginServer, ToggleThemeServer};
use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_hydrated::use_hydrated_context;

#[component]
pub fn Header() -> impl IntoView {
    let profile_state = use_hydrated_context::<ProfileState>();
    let theme_state = use_hydrated_context::<ThemeState>();
    let toggle_theme = ServerAction::<ToggleThemeServer>::new();
    let toggle_login = ServerAction::<ToggleLoginServer>::new();

    // Update theme reactively when action completes
    Effect::new(move |_| {
        if let Some(Ok(new_state)) = toggle_theme.value().get() {
            theme_state.set(new_state);
        }
    });

    // Update login state reactively when action completes
    Effect::new(move |_| {
        if let Some(Ok(new_state)) = toggle_login.value().get() {
            profile_state.set(new_state);
        }
    });

    view! {
        <header class="top-nav">
            <h1>"Hydrate Showcase"</h1>
            <div class="controls">
                <ActionForm action=toggle_theme>
                    <button type="submit" class="btn btn-secondary">
                        "Switch to "
                        {move || if theme_state.get().0 == "dark" { "Light" } else { "Dark" }}
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
