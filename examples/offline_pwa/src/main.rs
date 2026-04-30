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

    use leptos_hydrated::HydratedRouterExt;
    let app = Router::new()
        .route("/sw.js", get(sw_handler))
        .route("/manifest.json", get(manifest_handler))
        .route("/offline.html", get(offline_handler))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .hydrated()
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
    let sw = include_str!("../public/sw.js").to_string();
    let version = get_version();

    // Helper to detect if a filename is likely hashed (e.g. name.hash.js)
    let is_hashed = |name: &str| name.chars().filter(|&c| c == '.').count() > 1;

    // 1. Resolve all assets in the pkg directory to support code splitting
    let mut asset_paths = Vec::new();
    let pkg_path = std::path::PathBuf::from(options.site_root.as_ref())
        .join(options.site_pkg_dir.as_ref());

    if let Ok(entries) = std::fs::read_dir(&pkg_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    let is_asset = file_name.ends_with(".js")
                        || file_name.ends_with(".wasm")
                        || file_name.ends_with(".css");
                    let is_map = file_name.ends_with(".map");

                    if is_asset && !is_map {
                        if is_hashed(&file_name) {
                            asset_paths.push(format!("/{}/{}", options.site_pkg_dir, file_name));
                        } else {
                            asset_paths.push(format!("/{}/{}?v={}", options.site_pkg_dir, file_name, version));
                        }
                    }
                }
            }
        }
    }

    // Fallback if no assets found
    if asset_paths.is_empty() {
        asset_paths.push(format!("/{}/{}.js?v={}", options.site_pkg_dir, options.output_name, version));
        asset_paths.push(format!("/{}/{}.wasm?v={}", options.site_pkg_dir, options.output_name, version));
    }

    let assets_string = asset_paths
        .iter()
        .map(|p| format!("'{}',", p))
        .collect::<Vec<_>>()
        .join("\n");

    let sw = sw.replace("{{VERSION}}", &version);
    let sw = sw.replace("'{{ASSETS}}',", &assets_string);
    let sw = sw.replace("{{OUTPUT_NAME}}", &options.output_name);

    (
        [
            (axum::http::header::CONTENT_TYPE, "application/javascript"),
            (
                axum::http::header::CACHE_CONTROL,
                "no-cache, no-store, must-revalidate",
            ),
        ],
        sw,
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
    let mut html = include_str!("../public/offline.html").to_string();
    let version = get_version();

    // Helper to detect if a filename is likely hashed (e.g. name.hash.js)
    let is_hashed = |name: &str| name.chars().filter(|&c| c == '.').count() > 1;

    // Dynamically find the main bundle names (including hashes)
    let mut js_name = format!("{}.js", options.output_name);
    let mut wasm_name = format!("{}.wasm", options.output_name);
    let mut css_name = format!("{}.css", options.output_name);

    let pkg_path = std::path::PathBuf::from(options.site_root.as_ref())
        .join(options.site_pkg_dir.as_ref());

    if let Ok(entries) = std::fs::read_dir(&pkg_path) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with(options.output_name.as_ref()) && !name.ends_with(".map") {
                    if name.ends_with(".js") { 
                        // Prefer hashed names
                        if is_hashed(&name) || !is_hashed(&js_name) {
                            js_name = name;
                        }
                    }
                    else if name.ends_with(".wasm") { 
                        if is_hashed(&name) || !is_hashed(&wasm_name) {
                            wasm_name = name;
                        }
                    }
                    else if name.ends_with(".css") { 
                        if is_hashed(&name) || !is_hashed(&css_name) {
                            css_name = name;
                        }
                    }
                }
            }
        }
    }

    let js_path = if is_hashed(&js_name) {
        format!("/{}/{}", options.site_pkg_dir, js_name)
    } else {
        format!("/{}/{}?v={}", options.site_pkg_dir, js_name, version)
    };

    let wasm_path = if is_hashed(&wasm_name) {
        format!("/{}/{}", options.site_pkg_dir, wasm_name)
    } else {
        format!("/{}/{}?v={}", options.site_pkg_dir, wasm_name, version)
    };

    let css_path = if is_hashed(&css_name) {
        format!("/{}/{}", options.site_pkg_dir, css_name)
    } else {
        format!("/{}/{}?v={}", options.site_pkg_dir, css_name, version)
    };

    html = html.replace("{{OUTPUT_NAME}}", &options.output_name);
    html = html.replace("{{VERSION}}", &version);
    html = html.replace("{{JS_PATH}}", &js_path);
    html = html.replace("{{WASM_PATH}}", &wasm_path);
    html = html.replace("{{CSS_PATH}}", &css_path);
    html = html.replace("{{PKG_DIR}}", &options.site_pkg_dir);

    (
        [
            (axum::http::header::CONTENT_TYPE, "text/html"),
            (
                axum::http::header::CACHE_CONTROL,
                "no-cache, no-store, must-revalidate",
            ),
        ],
        html,
    )
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
