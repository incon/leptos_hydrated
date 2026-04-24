use crate::components::TabPanel;
use crate::states::ReferralState;
use leptos::prelude::*;
use leptos_hydrated::use_hydrated;
use leptos_router::components::A;
use leptos_router::hooks::query_signal;

#[component]
pub fn ParamsTab(tab: &'static str) -> impl IntoView {
    let referral_state = use_hydrated::<ReferralState>();
    let (_, _set_ref_query) = query_signal::<String>("ref");

    view! {
        <TabPanel tab=tab>
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
                <A
                    href=move || {
                        if referral_state.get().0.is_some() {
                            "?tab=params".to_string()
                        } else {
                            "?tab=params&ref=HYDRATE20".to_string()
                        }
                    }
                    attr:class="btn btn-primary"
                >
                    {move || {
                        if referral_state.get().0.is_some() {
                            "Remove Ref Parameter"
                        } else {
                            "Add Ref Parameter"
                        }
                    }}
                </A>
            </div>
        </TabPanel>
    }
}
