#[cfg(feature = "ssr")]
use offline_pwa::app::get_version;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use offline_pwa::app::*;

    use axum::routing::get;

    let conf = get_configuration(Some("./examples/offline_pwa/Cargo.toml")).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .route("/sw.js", get(sw_handler))
        .route("/manifest.json", get(manifest_handler))
        .route("/offline.html", get(offline_handler))
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            || {
                leptos_hydrated::provide_hydration_context();
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(feature = "ssr")]
async fn sw_handler(
    axum::extract::State(options): axum::extract::State<leptos::config::LeptosOptions>,
) -> impl axum::response::IntoResponse {
    use std::sync::OnceLock;
    static SW_CONTENT: OnceLock<String> = OnceLock::new();

    let content = SW_CONTENT.get_or_init(|| {
        let mut sw = include_str!("../public/sw.js").to_string();
        let version = get_version();
        sw = sw.replace("{{VERSION}}", &version);
        sw.replace("{{OUTPUT_NAME}}", &options.output_name)
    });

    (
        [
            (axum::http::header::CONTENT_TYPE, "application/javascript"),
            (
                axum::http::header::CACHE_CONTROL,
                "no-cache, no-store, must-revalidate",
            ),
        ],
        content.clone(),
    )
}

#[cfg(feature = "ssr")]
async fn manifest_handler() -> impl axum::response::IntoResponse {
    use std::sync::OnceLock;
    static MANIFEST_CONTENT: OnceLock<String> = OnceLock::new();

    let content = MANIFEST_CONTENT.get_or_init(|| {
        let mut manifest = include_str!("../public/manifest.json").to_string();
        let version = get_version();
        manifest = manifest.replace("{{VERSION}}", &version);
        manifest
    });

    (
        [
            (axum::http::header::CONTENT_TYPE, "application/json"),
            (
                axum::http::header::CACHE_CONTROL,
                "no-cache, no-store, must-revalidate",
            ),
        ],
        content.clone(),
    )
}

#[cfg(feature = "ssr")]
async fn offline_handler(
    axum::extract::State(options): axum::extract::State<leptos::config::LeptosOptions>,
) -> impl axum::response::IntoResponse {
    use std::sync::OnceLock;
    static OFFLINE_CONTENT: OnceLock<String> = OnceLock::new();

    let content = OFFLINE_CONTENT.get_or_init(|| {
        let mut html = include_str!("../public/offline.html").to_string();
        let version = get_version();
        html = html.replace("{{OUTPUT_NAME}}", &options.output_name);
        html.replace("{{VERSION}}", &version)
    });

    (
        [
            (axum::http::header::CONTENT_TYPE, "text/html"),
            (
                axum::http::header::CACHE_CONTROL,
                "no-cache, no-store, must-revalidate",
            ),
        ],
        content.clone(),
    )
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
