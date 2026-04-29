use leptos::prelude::*;
use leptos_hydrated::*;
use serde::{Deserialize, Serialize};

/// The internal data for a secure user session.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct SecureData {
    pub balance: i32,
    pub tier: String,
}

/// A wrapper for the optional secure user data.
/// Using a newtype struct for the Option makes hydration clean and type-safe.
#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct SecureUserData(pub Option<SecureData>);

impl Hydratable for SecureUserData {
    fn initial() -> Self {
        SecureUserData(read_secure_user_data())
    }

    fn should_sync_on_client() -> bool {
        // Disable client-side synchronization for secure data.
        false
    }
}

#[ssr]
pub fn read_secure_user_data() -> Option<SecureData> {
    // Only check the HTTP-only cookie on the server.
    if let Some(token) = get_cookie("secret_token") {
        if token == "HYDRATED_SECRET_TOKEN" {
            return Some(SecureData {
                balance: 5000,
                tier: "Platinum".to_string(),
            });
        }
    }

    None
}

#[hydrated_server]
pub async fn login_secure() -> Result<SecureUserData, ServerFnError> {
    set_cookie(
        "secret_token",
        "HYDRATED_SECRET_TOKEN",
        "; path=/; HttpOnly; SameSite=Lax",
    );
    Ok(SecureUserData(read_secure_user_data()))
}

#[hydrated_server]
pub async fn logout_secure() -> Result<SecureUserData, ServerFnError> {
    set_cookie(
        "secret_token",
        "",
        "; path=/; HttpOnly; SameSite=Lax; Max-Age=0",
    );
    Ok(SecureUserData(read_secure_user_data()))
}
