use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewUserRequest {
    pub name: String,
    pub password: String,
    #[serde(default)]
    pub group: Option<String>,
}
