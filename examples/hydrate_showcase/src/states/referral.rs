use leptos_hydrated::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct ReferralState(pub Option<String>);

impl Hydratable for ReferralState {
    fn initial() -> Self {
        read_referral_state()
    }
}

pub fn read_referral_state() -> ReferralState {
    ReferralState(get_query_param("ref"))
}
