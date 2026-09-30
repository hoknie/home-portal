use serde::Deserialize;

#[derive(Deserialize)]
pub struct GroupRequest {
    #[serde(default)]
    pub group: Option<String>,
}
