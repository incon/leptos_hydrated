use leptos_hydrated::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct TabState(pub String);

impl Hydratable for TabState {
    fn initial() -> Self {
        TabState(get_query_param("tab").unwrap_or_else(|| "cookie".into()))
    }
}
