#[macro_export]
macro_rules! isomorphic {
    (state => $state:expr, hydrate => $hydrate:expr $(,)?) => {{
        #[cfg(feature = "ssr")]
        {
            $state
        }
        #[cfg(not(feature = "ssr"))]
        {
            $hydrate
        }
    }};
}

