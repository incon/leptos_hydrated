use leptos::prelude::*;
use leptos_hydrated::use_hydrated;
use crate::states::ProfileState;

#[component]
pub fn ProfileCard() -> impl IntoView {
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
