use axum::{
    body::Body,
    extract::Request,
    http,
    middleware::{self, Next},
    response::Response,
    Router,
};
use crate::core::InjectedStates;

/// Extension trait for `axum::Router` to easily add hydration support.
pub trait HydratedRouterExt {
    /// Injects the hydration logic middleware into the router.
    /// This middleware collects isomorphic state during rendering and injects it
    /// into the HTML response as a JSON script tag.
    fn hydrated(self) -> Self;
}

impl<S> HydratedRouterExt for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn hydrated(self) -> Self {
        self.layer(middleware::from_fn(inject_logic))
    }
}

/// Marker to verify that the hydration middleware is active.
#[derive(Clone)]
pub(crate) struct HydrationMiddlewareMarker;

async fn inject_logic(req: Request, next: Next) -> Response {
    use crate::helpers::HydrationStore;
    
    let (mut parts, body) = req.into_parts();
    
    let states = InjectedStates::default();
    let store = HydrationStore::new_from_parts(&parts);
    
    // Explicitly insert into parts extensions
    parts.extensions.insert(states.clone());
    parts.extensions.insert(store.clone());
    parts.extensions.insert(HydrationMiddlewareMarker);

    // Reconstruct request
    let req = Request::from_parts(parts, body);
    
    let res = next.run(req).await;

    // After rendering, check if we have states to inject
    let injected = states.0.lock().unwrap().clone();
    if injected.is_empty() {
        return res;
    }

    let (mut parts, body) = res.into_parts();

    // Only inject into HTML responses
    let is_html = parts
        .headers
        .get(http::header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap_or("").contains("text/html"))
        .unwrap_or(false);

    if !is_html {
        return Response::from_parts(parts, body);
    }

    // Prepare the script tag that sets the global variable
    let json_array = format!("[{}]", injected.join(","));
    let script = format!(
        "<script>window.__lh_data = {};</script>",
        json_array
    );

    // Use a stream-based approach to preserve streaming support.
    // We wrap the body and append our script at the end.
    use futures_util::StreamExt;
    let stream = body.into_data_stream().chain(futures_util::stream::once(async move {
        Ok::<_, axum::Error>(axum::body::Bytes::from(script))
    }));

    // Remove Content-Length because we are changing the body size and don't want to calculate it upfront
    parts.headers.remove(http::header::CONTENT_LENGTH);

    Response::from_parts(parts, Body::from_stream(stream))
}
