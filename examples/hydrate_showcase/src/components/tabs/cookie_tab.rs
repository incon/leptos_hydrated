use leptos::prelude::*;
use crate::components::{ProfileCard, TabPanel};

#[component]
pub fn CookieTab(tab: &'static str) -> impl IntoView {
    view! {
        <TabPanel tab=tab>
            <ProfileCard />
            <div class="card info-card">
                <h2>"Synchronous Cookie Hydration"</h2>
                <p>
                    "The user session data is stored in a cookie. The server reads and renders the authenticated UI on the first frame."
                </p>
                <p>
                    "Try logging in and then hard-refreshing (Cmd/Ctrl+R). You will notice zero flickering."
                </p>
            </div>
        </TabPanel>
    }
}
