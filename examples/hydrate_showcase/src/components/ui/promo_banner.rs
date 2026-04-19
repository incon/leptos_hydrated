use leptos::prelude::*;
use leptos_hydrated::use_hydrated;
use crate::states::ReferralState;

#[component]
pub fn PromoBanner() -> impl IntoView {
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
