use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_hydrated::{HydrateContext, use_hydrated};
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{StaticSegment, components::*, hooks::query_signal};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct TabState(pub String);

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct ReferralState(pub Option<String>);

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct ProfileState {
    pub theme: String,
    pub is_authenticated: bool,
    pub profile: Option<UserProfile>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct UserProfile {
    pub name: String,
    pub role: String,
    pub edits: u32,
}

// Helper to safely read cookies without web-sys feature flags
#[cfg(not(feature = "ssr"))]
fn get_cookie() -> String {
    js_sys::Reflect::get(&document(), &wasm_bindgen::JsValue::from_str("cookie"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

// Helper to safely write cookies without web-sys feature flags
#[cfg(not(feature = "ssr"))]
fn set_cookie(cookie: &str) {
    let _ = js_sys::Reflect::set(
        &document(),
        &wasm_bindgen::JsValue::from_str("cookie"),
        &wasm_bindgen::JsValue::from_str(cookie),
    );
}

// --- Granular State Readers ---

fn read_tab_state() -> TabState {
    #[cfg(feature = "ssr")]
    {
        use http::request::Parts;
        let mut active_tab = "cookie".to_string();
        if let Some(parts) = use_context::<Parts>() {
            let query_str = parts.uri.query().unwrap_or_default();
            if let Some(tab) = query_str
                .split('&')
                .find(|s| s.starts_with("tab="))
                .and_then(|s| s.strip_prefix("tab="))
            {
                active_tab = tab.to_string();
            }
        }
        TabState(active_tab)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let mut active_tab = "cookie".to_string();
        let query_str = window()
            .location()
            .search()
            .unwrap_or_default()
            .strip_prefix('?')
            .unwrap_or_default()
            .to_string();
        if let Some(tab) = query_str
            .split('&')
            .find(|s| s.starts_with("tab="))
            .and_then(|s| s.strip_prefix("tab="))
        {
            active_tab = tab.to_string();
        }
        TabState(active_tab)
    }
}

fn read_referral_state() -> ReferralState {
    #[cfg(feature = "ssr")]
    {
        use http::request::Parts;
        let mut referral = None;
        if let Some(parts) = use_context::<Parts>() {
            let query_str = parts.uri.query().unwrap_or_default();
            if let Some(r) = query_str
                .split('&')
                .find(|s| s.starts_with("ref="))
                .and_then(|s| s.strip_prefix("ref="))
            {
                referral = Some(r.to_string());
            }
        }
        ReferralState(referral)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let mut referral = None;
        let query_str = window()
            .location()
            .search()
            .unwrap_or_default()
            .strip_prefix('?')
            .unwrap_or_default()
            .to_string();
        if let Some(r) = query_str
            .split('&')
            .find(|s| s.starts_with("ref="))
            .and_then(|s| s.strip_prefix("ref="))
        {
            referral = Some(r.to_string());
        }
        ReferralState(referral)
    }
}

fn read_profile_state() -> ProfileState {
    #[cfg(feature = "ssr")]
    {
        use http::request::Parts;
        let mut theme = "light".to_string();
        let mut profile = None;
        if let Some(parts) = use_context::<Parts>() {
            if let Some(cookie) = parts.headers.get("cookie").and_then(|c| c.to_str().ok()) {
                if cookie.contains("theme=dark") {
                    theme = "dark".to_string();
                }
                if let Some(sess_cookie) = cookie
                    .split("; ")
                    .find(|s| s.starts_with("session="))
                    .and_then(|s| s.strip_prefix("session="))
                {
                    if let Ok(decoded) = urlencoding::decode(sess_cookie) {
                        if let Ok(parsed) = serde_json::from_str::<UserProfile>(&decoded) {
                            profile = Some(parsed);
                        }
                    }
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
    #[cfg(not(feature = "ssr"))]
    {
        let mut theme = "light".to_string();
        let mut profile = None;
        let cookie = get_cookie();
        if cookie.contains("theme=dark") {
            theme = "dark".to_string();
        }
        if let Some(sess_cookie) = cookie
            .split("; ")
            .find(|s| s.starts_with("session="))
            .and_then(|s| s.strip_prefix("session="))
        {
            if let Ok(decoded) = urlencoding::decode(sess_cookie) {
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
}

// --- Granular Server Functions ---

#[server]
pub async fn fetch_tab_state() -> Result<TabState, ServerFnError> {
    use http::request::Parts;
    let mut state = read_tab_state();
    if let Some(parts) = use_context::<Parts>() {
        if let Some(referer) = parts.headers.get("referer").and_then(|r| r.to_str().ok()) {
            if let Ok(url) = http::Uri::try_from(referer) {
                if let Some(query_str) = url.query() {
                    if let Some(tab) = query_str
                        .split('&')
                        .find(|s| s.starts_with("tab="))
                        .and_then(|s| s.strip_prefix("tab="))
                    {
                        state.0 = tab.to_string();
                    }
                }
            }
        }
    }
    Ok(state)
}

#[server]
pub async fn fetch_referral_state() -> Result<ReferralState, ServerFnError> {
    use http::request::Parts;
    let mut state = read_referral_state();
    if let Some(parts) = use_context::<Parts>() {
        if let Some(referer) = parts.headers.get("referer").and_then(|r| r.to_str().ok()) {
            if let Ok(url) = http::Uri::try_from(referer) {
                if let Some(query_str) = url.query() {
                    if let Some(r) = query_str
                        .split('&')
                        .find(|s| s.starts_with("ref="))
                        .and_then(|s| s.strip_prefix("ref="))
                    {
                        state.0 = Some(r.to_string());
                    }
                }
            }
        }
    }
    Ok(state)
}

#[server]
pub async fn fetch_profile_state() -> Result<ProfileState, ServerFnError> {
    Ok(read_profile_state())
}

/// Updates the user profile in the session cookie.
#[server]
pub async fn update_profile(name: String, role: String) -> Result<UserProfile, ServerFnError> {
    use http::request::Parts;
    use leptos_axum::ResponseOptions;

    // 1. Read existing profile from cookie to get the current edit count
    let mut current_profile = None;
    if let Some(parts) = use_context::<Parts>() {
        if let Some(cookie) = parts.headers.get("cookie").and_then(|c| c.to_str().ok()) {
            if let Some(sess_cookie) = cookie
                .split("; ")
                .find(|s| s.starts_with("session="))
                .and_then(|s| s.strip_prefix("session="))
            {
                if let Ok(decoded) = urlencoding::decode(sess_cookie) {
                    if let Ok(parsed) = serde_json::from_str::<UserProfile>(&decoded) {
                        current_profile = Some(parsed);
                    }
                }
            }
        }
    }

    // 2. Ensure the user is "authenticated" (has a profile)
    let current_edits = match current_profile {
        Some(p) => p.edits,
        None => return Err(ServerFnError::Args("Not authenticated".to_string())),
    };

    let profile = UserProfile {
        name,
        role,
        edits: current_edits + 1,
    };

    // 3. Save the updated profile back to the session cookie
    if let Ok(json) = serde_json::to_string(&profile) {
        if let Some(response_options) = use_context::<ResponseOptions>() {
            response_options.insert_header(
                http::header::SET_COOKIE,
                http::HeaderValue::from_str(&format!(
                    "session={}; path=/; max-age=31536000",
                    urlencoding::encode(&json)
                ))
                .unwrap(),
            );
        }
    }

    Ok(profile)
}

#[component]
fn PromoBanner() -> impl IntoView {
    let state = use_hydrated::<ReferralState>();

    view! {
        <div
            class="promo-banner"
            style=move || if state.get().0.is_some() { "display: flex" } else { "display: none" }
        >
            <span class="promo-tag">"EXCLUSIVE OFFER"</span>
            <span class="promo-text">
                "Welcome! You have successfully hydrated the "
                <strong>"ref=" {move || state.get().0.unwrap_or_default()}</strong> " parameter."
            </span>
        </div>
    }
}

/// A helper component to keep hydrated signals in sync with the URL.
#[component]
fn SyncHydratedState() -> impl IntoView {
    let tab_state = use_hydrated::<TabState>();
    let referral_state = use_hydrated::<ReferralState>();

    let (tab_query, _) = query_signal::<String>("tab");
    let (ref_query, _) = query_signal::<String>("ref");

    // Sync tabs
    Effect::new(move |_| {
        let current_tab = tab_query.get().unwrap_or_else(|| "cookie".to_string());
        if tab_state.get_untracked().0 != current_tab {
            tab_state.set(TabState(current_tab));
        }
    });

    // Sync referral
    Effect::new(move |_| {
        let current_ref = ref_query.get();
        if referral_state.get_untracked().0 != current_ref {
            referral_state.set(ReferralState(current_ref));
        }
    });
}

#[component]
fn ProfileContext(children: Children) -> impl IntoView {
    view! {
        <HydrateContext ssr_value=read_profile_state fetcher=fetch_profile_state>
            {children()}
        </HydrateContext>
    }
}

#[component]
fn TabContext(children: Children) -> impl IntoView {
    view! {
        <HydrateContext ssr_value=read_tab_state fetcher=fetch_tab_state>
            {children()}
        </HydrateContext>
    }
}

#[component]
fn ReferralContext(children: Children) -> impl IntoView {
    view! {
        <HydrateContext ssr_value=read_referral_state fetcher=fetch_referral_state>
            {children()}
        </HydrateContext>
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/hydrate_showcase.css" />
        <Title text="Hydrate Showcase" />

        <ProfileContext>
            <Router>
                <TabContext>
                    <ReferralContext>
                        <SyncHydratedState />
                        <ThemeWrapper>
                            <PromoBanner />
                            <Routes fallback=|| "Page not found.".into_view()>
                                <Route path=StaticSegment("") view=HomePage />
                            </Routes>
                        </ThemeWrapper>
                    </ReferralContext>
                </TabContext>
            </Router>
        </ProfileContext>
    }
}

/// Wraps the application to apply the global theme class synchronously
#[component]
fn ThemeWrapper(children: Children) -> impl IntoView {
    let state = use_hydrated::<ProfileState>();

    view! {
        <div class=move || format!("app-wrapper theme-{}", state.get().theme)>
            <main>{children()}</main>
        </div>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let profile_state = use_hydrated::<ProfileState>();
    let tab_state = use_hydrated::<TabState>();
    let referral_state = use_hydrated::<ReferralState>();
    let update_profile_action = ServerAction::<UpdateProfile>::new();

    // Sync the profile state when the update action succeeds
    Effect::new(move |_| {
        if let Some(Ok(new_profile)) = update_profile_action.value().get() {
            profile_state.update(|s| {
                s.is_authenticated = true;
                s.profile = Some(new_profile);
            });
        }
    });

    let (_, set_ref_query) = query_signal::<String>("ref");

    let toggle_theme = move |_| {
        profile_state.update(|s| {
            let new_theme = if s.theme == "dark" { "light" } else { "dark" };
            s.theme = new_theme.to_string();
            #[cfg(not(feature = "ssr"))]
            set_cookie(&format!("theme={}; path=/; max-age=31536000", new_theme));
        });
    };

    let toggle_login = move |_| {
        profile_state.update(|s| {
            if s.is_authenticated {
                s.is_authenticated = false;
                s.profile = None;
                #[cfg(not(feature = "ssr"))]
                set_cookie("session=; path=/; max-age=0");
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
                    set_cookie(&format!(
                        "session={}; path=/; max-age=31536000",
                        urlencoding::encode(&json)
                    ));
                }
            }
        });
        #[cfg(not(feature = "ssr"))]
        let _ = window().location().reload();
    };

    let toggle_ref = move |_| {
        if referral_state.get_untracked().0.is_some() {
            set_ref_query.set(None);
        } else {
            set_ref_query.set(Some("HYDRATE20".to_string()));
        }
    };

    view! {
        <header class="top-nav">
            <h1>"Hydrate Showcase"</h1>
            <div class="controls">
                <button class="btn btn-secondary" on:click=toggle_theme>
                    "Switch to "
                    {move || if profile_state.get().theme == "dark" { "Light" } else { "Dark" }}
                </button>
                <button class="btn btn-primary" on:click=toggle_login>
                    {move || {
                        if profile_state.get().is_authenticated { "Log Out" } else { "Log In" }
                    }}
                </button>
            </div>
        </header>

        <div class="tabs">
            <A
                href="?tab=cookie"
                attr:class=move || {
                    format!("tab-btn {}", if tab_state.get().0 == "cookie" { "active" } else { "" })
                }
            >
                "Cookie State"
            </A>
            <A
                href="?tab=params"
                attr:class=move || {
                    format!("tab-btn {}", if tab_state.get().0 == "params" { "active" } else { "" })
                }
            >
                "Parameter State"
            </A>
            <A
                href="?tab=reactivity"
                attr:class=move || {
                    format!(
                        "tab-btn {}",
                        if tab_state.get().0 == "reactivity" { "active" } else { "" },
                    )
                }
            >
                "Reactivity"
            </A>
        </div>

        <div class="dashboard-grid">
            <div
                class="params-content"
                style=move || {
                    if tab_state.get().0 == "params" {
                        "display: contents"
                    } else {
                        "display: none"
                    }
                }
            >
                <div class="card info-card">
                    <h2>"URL Parameter Hydration"</h2>
                    <p>
                        "This page demonstrates state driven entirely by the URL. No JavaScript is required for the initial render."
                    </p>
                    <p>
                        "When you click the button below, then hard refresh (Cmd/Ctrl+R), the page will reload with a new parameter. Because of "
                        <strong>"leptos_hydrate"</strong>
                        ", the server knows exactly what to render immediately."
                    </p>
                    <button class="btn btn-primary" on:click=toggle_ref>
                        {move || {
                            if referral_state.get().0.is_some() {
                                "Remove Ref Parameter"
                            } else {
                                "Add Ref Parameter"
                            }
                        }}
                    </button>
                </div>
            </div>

            <div
                class="reactivity-content"
                style=move || {
                    if tab_state.get().0 == "reactivity" {
                        "display: contents"
                    } else {
                        "display: none"
                    }
                }
            >
                <div class="card reactivity-card">
                    <h2>"Reactive Form Updates"</h2>
                    {move || {
                        let s = profile_state.get();
                        if s.is_authenticated {
                            view! {
                                <>
                                    <p>
                                        "Update your profile data using this form. The default values are pre-populated from the "
                                        <strong>"HydrateContext"</strong> " state."
                                    </p>
                                    <UpdateProfileForm
                                        action=update_profile_action
                                        profile=s.profile
                                    />
                                    <p class="note">
                                        "After submitting, the state will be updated reactively in the UI and synchronized with your session cookie."
                                    </p>
                                </>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="guest-state">
                                    <p>"You must be logged in to edit your profile."</p>
                                    <button class="btn btn-primary" on:click=toggle_login>
                                        "Log In Now"
                                    </button>
                                </div>
                            }
                                .into_any()
                        }
                    }}
                </div>
            </div>

            <div
                class="cookie-content"
                style=move || {
                    if tab_state.get().0 == "cookie"
                        || (tab_state.get().0 != "params" && tab_state.get().0 != "reactivity")
                    {
                        "display: contents"
                    } else {
                        "display: none"
                    }
                }
            >
                <ProfileCard />
                <div class="card info-card">
                    <h2>"Synchronous Cookie Hydration"</h2>
                    <p>
                        "The user session data is stored in a cookie. The server reads and renders the authenticated UI on the first frame."
                    </p>
                    <p>
                        "Try logging in and then hard-refreshing (Cmd/Ctrl+R). You will notice zero flickering or blanking."
                    </p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn UpdateProfileForm(
    action: ServerAction<UpdateProfile>,
    profile: Option<UserProfile>,
) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <div class="form-group">
                <label for="name">"Name"</label>
                <input
                    type="text"
                    name="name"
                    id="name"
                    value=profile.as_ref().map(|p| p.name.clone()).unwrap_or_default()
                    placeholder="Enter your name"
                />
            </div>
            <div class="form-group">
                <label for="role">"Role"</label>
                <input
                    type="text"
                    name="role"
                    id="role"
                    value=profile.as_ref().map(|p| p.role.clone()).unwrap_or_default()
                    placeholder="Enter your role"
                />
            </div>
            <button type="submit" class="btn btn-primary">
                "Update Profile"
            </button>
        </ActionForm>
    }
}

#[component]
fn ProfileCard() -> impl IntoView {
    let state = use_hydrated::<ProfileState>();

    view! {
        <div class="card profile-card">
            <h2>"User Profile"</h2>
            <div class="card-content">
                {move || {
                    let s = state.get();
                    if let Some(profile) = s.profile.clone() {
                        view! {
                            <div class="profile-data">
                                <p class="greeting">
                                    {move || format!("Welcome back, {}!", profile.name)}
                                </p>
                                <p>
                                    <span>"Role: "</span>
                                    <span class="badge">{profile.role}</span>
                                </p>
                                <p>
                                    <span>"Session Edits: "</span>
                                    {profile.edits}
                                </p>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="guest-state">
                                <p>"You are currently browsing as a guest."</p>
                                <p>"Log in to view your secure profile data."</p>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}
