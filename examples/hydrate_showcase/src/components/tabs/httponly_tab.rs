use crate::components::TabPanel;
use crate::states::{LoginSecure, LogoutSecure, SecureUserData};
use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_hydrated::client_only;
use leptos_hydrated::*;

#[component]
pub fn HttpOnlyTab(tab: &'static str) -> impl IntoView {
    let secure_state = use_hydrated::<SecureUserData>();
    let login_action = ServerAction::<LoginSecure>::new();
    let logout_action = ServerAction::<LogoutSecure>::new();

    // Reload the page after login/logout to see the HTTP-only cookie in action
    Effect::new(move |_| {
        if login_action.value().get().is_some() || logout_action.value().get().is_some() {
            client_only! {
                use leptos::prelude::window;
                let _ = window().location().reload();
            };
        }
    });

    view! {
        <TabPanel tab=tab>
            <div class="card httponly-card">
                <h2>"HTTP-only Cookie State"</h2>
                <p>
                    "This state is managed via an " <strong>"HTTP-only"</strong> " cookie. The client "
                    <em>"cannot"</em> " read or modify this cookie via JavaScript."
                </p>

                <div class="secure-box">
                    {move || {
                        let state = secure_state.get();
                        if state.tier != "Guest" {
                            view! {
                                <div class="token-display">
                                    <div class="status-container">
                                        <span class="label">"Account Tier:"</span>
                                        <span class="tier-badge">{state.tier}</span>
                                    </div>
                                    <span class="label">"Current Balance:"</span>
                                    <span class="balance-display">{format!("${}.00", state.balance)}</span>

                                    <ActionForm action=logout_action>
                                        <button type="submit" class="btn btn-danger"
                                            style="margin-top: 1rem; align-self: flex-start;">
                                            "Secure Log Out"
                                        </button>
                                    </ActionForm>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="login-prompt">
                                    <p>"No secure session active. Your balance is protected by HTTP-only cookies."</p>
                                    <ActionForm action=login_action>
                                        <button type="submit" class="btn btn-primary">"Secure Log In"</button>
                                    </ActionForm>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                <p class="note">
                    "When you log in securely, the server sets a secure, HTTP-only cookie. On subsequent requests, the server uses this cookie to hydrate the "
                    <code>"SecureUserData"</code> " state."
                </p>
            </div>
        </TabPanel>
    }
}
