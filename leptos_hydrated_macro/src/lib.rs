use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// A macro to gate a function's body for client-side WASM execution only.
///
/// On the client (WASM + no SSR feature), the full function body is preserved.
/// On all other platforms (including SSR), the body is replaced with an empty block.
#[proc_macro_attribute]
pub fn hydrate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    
    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            #[cfg(all(target_arch = "wasm32", not(feature = "ssr")))]
            {
                #block
            }
            
            #[cfg(not(all(target_arch = "wasm32", not(feature = "ssr"))))]
            {
                // Isomorphic placeholder
                Default::default()
            }
        }
    };

    TokenStream::from(expanded)
}

/// A macro to gate a function's body for server-side execution only.
///
/// On the server (SSR feature enabled), the full function body is preserved.
/// On the client (not SSR), the body is replaced with a default value.
#[proc_macro_attribute]
pub fn ssr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    
    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            #[cfg(feature = "ssr")]
            {
                #block
            }
            
            #[cfg(not(feature = "ssr"))]
            {
                // Isomorphic placeholder
                Default::default()
            }
        }
    };

    TokenStream::from(expanded)
}

/// A wrapper around `#[server]` that automatically uses the zero-weight `BrowserJson` codec.
///
/// This eliminates the `serde_json` dependency on the client, significantly reducing bundle size.
///
/// Usage:
/// ```rust
/// #[hydrated_server]
/// pub async fn my_server_fn(data: MyData) -> Result<MyResponse, ServerFnError> {
///    // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn hydrated_server(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let attr = proc_macro2::TokenStream::from(attr);
    
    let comma = if attr.is_empty() {
        quote! {}
    } else {
        quote! { , }
    };

    let expanded = quote! {
        #[server(#attr #comma input = ::leptos::server_fn::codec::PostUrl, output = ::leptos_hydrated::codec::BrowserJsonPost)]
        #input
    };

    TokenStream::from(expanded)
}
