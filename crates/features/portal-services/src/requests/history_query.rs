use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default)]
    pub range: Option<String>,
}
