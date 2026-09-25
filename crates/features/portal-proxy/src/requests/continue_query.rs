use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct ContinueQuery {
    #[serde(default)]
    pub to: Option<String>,
}
