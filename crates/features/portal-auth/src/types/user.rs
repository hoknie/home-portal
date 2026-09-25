use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub name: String,
    pub password_hash: String,
}
