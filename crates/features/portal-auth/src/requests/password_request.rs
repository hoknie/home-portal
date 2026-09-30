use serde::Deserialize;

#[derive(Deserialize)]
pub struct PasswordRequest {
    pub password: String,
    #[serde(default)]
    pub current_password: Option<String>,
}
