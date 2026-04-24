use leptos::prelude::*;
use leptos_hydrated::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct ProfileState {
    pub theme: String,
    pub is_authenticated: bool,
    pub profile: Option<UserProfile>,
}

impl ProfileState {
    pub fn toggle_theme(state: RwSignal<Self>) {
        state.update(|s| {
            let new_theme = if s.theme == "dark" { "light" } else { "dark" };
            s.theme = new_theme.to_string();
            #[cfg(not(feature = "ssr"))]
            set_cookie("theme", &new_theme, "; path=/; max-age=31536000");
        });
    }

    pub fn toggle_login(state: RwSignal<Self>) {
        state.update(|s| {
            if s.is_authenticated {
                s.is_authenticated = false;
                s.profile = None;
                #[cfg(not(feature = "ssr"))]
                set_cookie("session", "", "; path=/; max-age=0");
            } else {
                s.is_authenticated = true;
                let profile = UserProfile {
                    name: "Leptos Developer".to_string(),
                    role: "Systems Administrator".to_string(),
                    edits: 42,
                };
                s.profile = Some(profile.clone());
                #[cfg(not(feature = "ssr"))]
                if let Ok(json) = serde_json::to_string(&profile) {
                    set_cookie(
                        "session",
                        &urlencoding::encode(&json),
                        "; path=/; max-age=31536000",
                    );
                }
            }
        });
        #[cfg(not(feature = "ssr"))]
        {
            use leptos::prelude::window;
            let _ = window().location().reload();
        }
    }
}

impl Hydratable for ProfileState {
    fn initial() -> Self {
        read_profile_state()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct UserProfile {
    pub name: String,
    pub role: String,
    pub edits: u32,
}

pub fn read_profile_state() -> ProfileState {
    let mut theme = "light".to_string();
    let mut profile = None;

    if let Some(cookie) = get_cookie("theme") {
        if cookie == "dark" {
            theme = "dark".to_string();
        }
    }

    if let Some(sess_cookie) = get_cookie("session") {
        if let Ok(decoded) = urlencoding::decode(&sess_cookie) {
            if let Ok(parsed) = serde_json::from_str::<UserProfile>(&decoded) {
                profile = Some(parsed);
            }
        }
    }

    let is_authenticated = profile.is_some();
    ProfileState {
        theme,
        is_authenticated,
        profile,
    }
}

#[server]
pub async fn fetch_profile_state() -> Result<ProfileState, ServerFnError> {
    Ok(read_profile_state())
}

#[server]
pub async fn update_profile(name: String, role: String) -> Result<UserProfile, ServerFnError> {
    let current_profile = get_cookie("session")
        .and_then(|s| urlencoding::decode(&s).ok().map(|d| d.into_owned()))
        .and_then(|decoded| serde_json::from_str::<UserProfile>(&decoded).ok());

    let current_edits = match current_profile {
        Some(p) => p.edits,
        None => return Err(ServerFnError::Args("Not authenticated".to_string())),
    };

    let profile = UserProfile {
        name,
        role,
        edits: current_edits + 1,
    };

    if let Ok(json) = serde_json::to_string(&profile) {
        set_cookie(
            "session",
            &urlencoding::encode(&json),
            "; path=/; max-age=31536000",
        );
    }

    Ok(profile)
}
