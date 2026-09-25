use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct DashboardQuery {
    #[serde(default)]
    pub all: bool,
}
