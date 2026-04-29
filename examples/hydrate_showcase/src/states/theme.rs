use leptos::prelude::*;

use leptos_hydrated::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct ThemeState(pub String);

impl ThemeState {
    pub fn toggle(state: RwSignal<Self>) {
        state.update(|s| {
            let new_theme = if s.0 == "dark" { "light" } else { "dark" };
            s.0 = new_theme.to_string();
            #[cfg(not(feature = "ssr"))]
            {
                set_cookie("theme", &new_theme, "; path=/; max-age=31536000");
            }
        });
    }
}

impl Hydratable for ThemeState {
    fn initial() -> Self {
        read_theme_state()
    }
}

pub fn read_theme_state() -> ThemeState {
    let mut theme = "light".to_string();
    if let Some(cookie) = get_cookie("theme") {
        if cookie == "dark" {
            theme = "dark".to_string();
        }
    }
    ThemeState(theme)
}

#[hydrated_server]
pub async fn toggle_theme_server() -> Result<ThemeState, ServerFnError> {
    let theme = get_cookie("theme").unwrap_or_else(|| "light".to_string());
    let new_theme = if theme == "dark" { "light" } else { "dark" };
    set_cookie("theme", &new_theme, "; path=/; max-age=31536000");
    Ok(read_theme_state())
}
