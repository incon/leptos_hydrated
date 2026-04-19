use leptos::prelude::*;
use leptos::form::ActionForm;
use crate::states::{UpdateProfile, UserProfile};

#[component]
pub fn UpdateProfileForm(
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
