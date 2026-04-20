
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use offline_pwa::app::*;

    use axum::routing::get;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .route("/sw.js", get(sw_handler))
        .route("/manifest.json", get(manifest_handler))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
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
        let version = if cfg!(debug_assertions) {
            // In dev, use a timestamp to ensure the SW updates on every server restart
            format!("dev-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
        } else {
            // In prod, use the Cargo version
            env!("CARGO_PKG_VERSION").to_string()
        };
        sw = sw.replace("{{VERSION}}", &version);
        sw.replace("{{OUTPUT_NAME}}", &options.output_name)
    });

    (
        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
        content.clone(),
    )
}

#[cfg(feature = "ssr")]
async fn manifest_handler() -> impl axum::response::IntoResponse {
    use std::sync::OnceLock;
    static MANIFEST_CONTENT: OnceLock<String> = OnceLock::new();

    let content = MANIFEST_CONTENT.get_or_init(|| {
        let manifest = include_str!("../public/manifest.json").to_string();
        let version = if cfg!(debug_assertions) {
            format!("dev-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
        } else {
            env!("CARGO_PKG_VERSION").to_string()
        };
        manifest.replace("{{VERSION}}", &version)
    });

    (
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        content.clone(),
    )
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
