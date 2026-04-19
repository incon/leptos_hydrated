use leptos::prelude::*;
use leptos_hydrated::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct TabState(pub String);

impl Hydratable for TabState {
    fn initial() -> Self {
        read_tab_state()
    }
    async fn fetch() -> Option<Result<Self, ServerFnError>> {
        Some(fetch_tab_state().await)
    }
}

pub fn read_tab_state() -> TabState {
    TabState(get_query_param("tab").unwrap_or_else(|| "cookie".into()))
}

#[server]
pub async fn fetch_tab_state() -> Result<TabState, ServerFnError> {
    let mut state = read_tab_state();
    if let Some(tab) = get_referer_query_param("tab") {
        state.0 = tab;
    }
    Ok(state)
}
