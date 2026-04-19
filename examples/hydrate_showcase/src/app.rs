use leptos::prelude::*;
use leptos_hydrated::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{StaticSegment, components::*};

use crate::components::*;
use crate::states::*;



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
            <HydrateContext<ProfileState>>
                <HydrateContext<SecureUserData>>
                    <Router>
                        <HydrateContext<TabState>>
                            <HydrateContext<ReferralState>>
                                <ThemeWrapper>
                                    <PromoBanner />
                                    <Routes fallback=|| "Page not found.".into_view()>
                                        <Route path=StaticSegment("") view=HomePage />
                                    </Routes>
                                </ThemeWrapper>
                            </HydrateContext<ReferralState>>
                        </HydrateContext<TabState>>
                    </Router>
                </HydrateContext<SecureUserData>>
            </HydrateContext<ProfileState>>
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
