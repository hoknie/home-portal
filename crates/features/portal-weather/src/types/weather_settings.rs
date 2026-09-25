use serde::Deserialize;

use super::Units;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeatherSettings {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub units: Units,
    #[serde(default = "WeatherSettings::default_days")]
    pub days: u8,
}

impl WeatherSettings {
    pub const DAYS: std::ops::RangeInclusive<u8> = 1..=7;
    pub const DEFAULT_DAYS: u8 = 3;

    pub fn default_days() -> u8 {
        Self::DEFAULT_DAYS
    }

    pub fn timezone_or(&self, portal: &str) -> String {
        self.timezone.clone().unwrap_or_else(|| portal.to_string())
    }
}
