use crate::app::ReferralState;
use leptos::prelude::*;

#[component]
pub fn ParamsTab(
    referral_state: RwSignal<ReferralState>,
    toggle_ref: Callback<leptos::ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <div class="card info-card">
            <h2>"URL Parameter Hydration"</h2>
            <p>
                "This page demonstrates state driven entirely by the URL. No JavaScript is required for the initial render."
            </p>
            <p>
                "When you click the button below, then hard refresh (Cmd/Ctrl+R), the page will reload with a new parameter. Because of "
                <strong>"leptos_hydrated"</strong>
                ", the server knows exactly what to render immediately."
            </p>
            <button class="btn btn-primary" on:click=move |ev| toggle_ref.run(ev)>
                {move || {
                    if referral_state.get().0.is_some() {
                        "Remove Ref Parameter"
                    } else {
                        "Add Ref Parameter"
                    }
                }}
            </button>
        </div>
    }
}
