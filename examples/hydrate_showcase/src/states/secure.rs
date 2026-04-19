use leptos::prelude::*;
use leptos_hydrated::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct SecureUserData {
    pub balance: i32,
    pub tier: String,
}

impl Hydratable for SecureUserData {
    fn initial() -> Self {
        read_secure_user_data()
    }
}

pub fn read_secure_user_data() -> SecureUserData {
    let mut balance = 0;
    let mut tier = "Guest".to_string();

    if let Some(token) = get_cookie("secret_token") {
        if token == "HYDRATED_SECRET_TOKEN" {
            balance = 5000;
            tier = "Platinum".to_string();
        }
    }

    SecureUserData { balance, tier }
}

#[server]
pub async fn login_secure() -> Result<(), ServerFnError> {
    set_cookie(
        "secret_token",
        "HYDRATED_SECRET_TOKEN",
        "; path=/; HttpOnly; SameSite=Lax",
    );
    Ok(())
}

#[server]
pub async fn logout_secure() -> Result<(), ServerFnError> {
    set_cookie(
        "secret_token",
        "",
        "; path=/; HttpOnly; SameSite=Lax; Max-Age=0",
    );
    Ok(())
}
