use leptos::prelude::*;

fn main() {
    #[cfg(feature = "hydrate")]
    {
        let _ = is_hydrating();
    }
}
