#[macro_export]
macro_rules! isomorphic {
    (server => $server:expr, client => $client:expr $(,)?) => {{
        #[cfg(feature = "ssr")]
        {
            $server
        }
        #[cfg(not(feature = "ssr"))]
        {
            $client
        }
    }};
}

/// Executes the given block only on the server.
/// Returns the result of the block on the server, or `()` in the browser.
/// This is useful for side-effects where you don't need an `Option`.
#[macro_export]
macro_rules! server_only {
    ($($t:tt)*) => {
        {
            #[cfg(feature = "ssr")]
            { $($t)*; }
            ()
        }
    }
}

/// Executes the given block only in the browser.
/// Returns the result of the block in the browser, or `()` on the server.
/// This is useful for side-effects where you don't need an `Option`.
#[macro_export]
macro_rules! client_only {
    ($($t:tt)*) => {
        {
            #[cfg(not(feature = "ssr"))]
            {
                $($t)*
            }
            #[cfg(feature = "ssr")]
            {
                ()
            }
        }
    };
}
