#[macro_export]
macro_rules! isomorphic {
    (state => $state:expr, hydrate => $hydrate:expr $(,)?) => {{
        #[cfg(feature = "ssr")]
        {
            $state
        }
        #[cfg(all(not(feature = "ssr"), target_arch = "wasm32"))]
        {
            $hydrate
        }
        #[cfg(all(not(feature = "ssr"), not(target_arch = "wasm32")))]
        {
            $state
        }
    }};
}
