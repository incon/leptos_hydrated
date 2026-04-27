#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use hydrate_showcase::app::*;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    let conf = get_configuration(Some("examples/hydrate_showcase/Cargo.toml")).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    async fn inject_logic(
        mut req: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        use leptos_hydrated::InjectedStates;
        let states = InjectedStates::default();
        req.extensions_mut().insert(states.clone());
        
        let res = next.run(req).await;
        
        let injected = states.0.lock().unwrap().clone();
        if injected.is_empty() {
            return res;
        }
        
        let (mut parts, body) = res.into_parts();
        
        let is_html = parts.headers.get(axum::http::header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or("").contains("text/html"))
            .unwrap_or(false);
            
        if !is_html {
            return axum::response::Response::from_parts(parts, body);
        }
        
        let bytes = match axum::body::to_bytes(body, usize::MAX).await {
            Ok(b) => b,
            Err(_) => return axum::response::Response::from_parts(parts, axum::body::Body::empty()),
        };
        
        let mut html = String::from_utf8_lossy(&bytes).into_owned();
        
        let json_array = format!("[{}]", injected.join(","));
        let script = format!("<script id=\"__lh_data\" type=\"application/json\">{}</script>", json_array);
        
        if let Some(idx) = html.find("</body>") {
            html.insert_str(idx, &script);
        } else {
            html.push_str(&script);
        }
        
        parts.headers.insert(
            axum::http::header::CONTENT_LENGTH,
            axum::http::HeaderValue::from_str(&html.len().to_string()).unwrap(),
        );
        
        axum::response::Response::from_parts(parts, axum::body::Body::from(html))
    }

    let app = Router::new()
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
        .layer(axum::middleware::from_fn(inject_logic))
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

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
