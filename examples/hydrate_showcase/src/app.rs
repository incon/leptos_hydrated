use crate::components::*;
use crate::states::*;
use leptos::prelude::*;
use leptos_hydrated::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{components::*, StaticSegment};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/hydrate_showcase.css" />
        <Title text="Hydrate Showcase" />

        <div id="app-root">
            <HydratedContext<ProfileState> global=true />
            <HydratedContext<SecureUserData>>
                <Router>
                    <HydratedContext<TabState>>
                        <HydratedContext<ReferralState>>
                            <ThemeWrapper>
                                <PromoBanner />
                                <Routes fallback=|| "Page not found.".into_view()>
                                    <Route path=StaticSegment("") view=HomePage />
                                </Routes>
                            </ThemeWrapper>
                        </HydratedContext<ReferralState>>
                    </HydratedContext<TabState>>
                </Router>
            </HydratedContext<SecureUserData>>
        </div>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <Header />
        <Tabs>
            <CookieTab tab="cookie" />
            <ParamsTab tab="params" />
            <ReactivityTab tab="reactivity" />
            <HttpOnlyTab tab="httponly" />
        </Tabs>
    }
}
