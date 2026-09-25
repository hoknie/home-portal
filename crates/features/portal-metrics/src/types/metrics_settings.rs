use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsSettings {
    #[serde(default)]
    pub disks: Option<Vec<String>>,
}
