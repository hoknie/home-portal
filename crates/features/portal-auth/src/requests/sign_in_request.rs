use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SignInRequest {
    pub name: String,
    pub password: String,
}
