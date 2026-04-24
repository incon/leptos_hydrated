use leptos::prelude::*;
use leptos::form::ActionForm;
use leptos_hydrated::use_hydrated;
use crate::states::{ProfileState, UpdateProfile, ToggleLoginServer};
use crate::components::{UpdateProfileForm, TabPanel};

#[component]
pub fn ReactivityTab(tab: &'static str) -> impl IntoView {
    let profile_state = use_hydrated::<ProfileState>();
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

    view! {
        <TabPanel tab=tab>
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
                        let toggle_login = ServerAction::<ToggleLoginServer>::new();
                        view! {
                            <div class="guest-state">
                                <p>"You must be logged in to edit your profile."</p>
                                <ActionForm action=toggle_login>
                                    <button type="submit" class="btn btn-primary">
                                        "Log In Now"
                                    </button>
                                </ActionForm>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </TabPanel>
    }
}
