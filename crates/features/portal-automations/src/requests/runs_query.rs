use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RunsQuery {
    #[serde(default)]
    pub automation: Option<String>,
    #[serde(default)]
    pub webhook: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}
