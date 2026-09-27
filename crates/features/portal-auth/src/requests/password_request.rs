use serde::Deserialize;

#[derive(Deserialize)]
pub struct PasswordRequest {
    pub password: String,
}
