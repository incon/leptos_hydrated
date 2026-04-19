use leptos::prelude::*;
use leptos_hydrated::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct ReferralState(pub Option<String>);

impl Hydratable for ReferralState {
    fn initial() -> Self {
        read_referral_state()
    }
    async fn fetch() -> Option<Self> {
        fetch_referral_state().await.ok()
    }
}

pub fn read_referral_state() -> ReferralState {
    ReferralState(get_query_param("ref"))
}

#[server]
pub async fn fetch_referral_state() -> Result<ReferralState, ServerFnError> {
    let mut state = read_referral_state();
    if let Some(r) = get_referer_query_param("ref") {
        state.0 = Some(r);
    }
    Ok(state)
}
