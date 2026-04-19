use leptos::prelude::*;
use leptos_router::{components::*, hooks::query_signal};
use leptos_hydrated::use_hydrated;
use crate::states::{TabState, ReferralState};

#[component]
pub fn TabPanel(tab: &'static str, children: Children) -> impl IntoView {
    let tab_state = use_hydrated::<TabState>();
    
    view! {
        <div
            class=format!("{}-content", tab)
            style=move || {
                let current = tab_state.get().0;
                if current == tab || (tab == "cookie" && (current != "params" && current != "reactivity" && current != "httponly")) {
                    "display: contents"
                } else {
                    "display: none"
                }
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn Tabs(children: Children) -> impl IntoView {
    let tab_state = use_hydrated::<TabState>();
    let referral_state = use_hydrated::<ReferralState>();

    let (tab_query, _) = query_signal::<String>("tab");
    let (ref_query, _) = query_signal::<String>("ref");

    // Sync tabs with URL
    Effect::new(move |_| {
        let current_tab = tab_query.get().unwrap_or_else(|| "cookie".to_string());
        if tab_state.get_untracked().0 != current_tab {
            tab_state.set(TabState(current_tab));
        }
    });

    // Sync referral with URL
    Effect::new(move |_| {
        let current_ref = ref_query.get();
        if referral_state.get_untracked().0 != current_ref {
            referral_state.set(ReferralState(current_ref));
        }
    });

    view! {
        <div class="tabs">
            <A
                href="?tab=cookie"
                attr:class=move || {
                    format!("tab-btn {}", if tab_state.get().0 == "cookie" { "active" } else { "" })
                }
            >
                "Cookie State"
            </A>
            <A
                href="?tab=params"
                attr:class=move || {
                    format!("tab-btn {}", if tab_state.get().0 == "params" { "active" } else { "" })
                }
            >
                "Parameter State"
            </A>
            <A
                href="?tab=reactivity"
                attr:class=move || {
                    format!(
                        "tab-btn {}",
                        if tab_state.get().0 == "reactivity" { "active" } else { "" },
                    )
                }
            >
                "Reactivity"
            </A>
            <A
                href="?tab=httponly"
                attr:class=move || {
                    format!("tab-btn {}", if tab_state.get().0 == "httponly" { "active" } else { "" })
                }
            >
                "HTTP-only"
            </A>
        </div>
        <div class="dashboard-grid">
            {children()}
        </div>
    }
}
