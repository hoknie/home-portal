use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarSettings {
    pub url: String,
    #[serde(default = "CalendarSettings::default_days")]
    pub days: u8,
    #[serde(default = "CalendarSettings::default_limit")]
    pub limit: u8,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub secret: Option<String>,
}

impl CalendarSettings {
    pub const DAYS: std::ops::RangeInclusive<u8> = 1..=31;
    pub const LIMIT: std::ops::RangeInclusive<u8> = 1..=50;
    pub const DEFAULT_DAYS: u8 = 7;
    pub const DEFAULT_LIMIT: u8 = 10;

    pub fn default_days() -> u8 {
        Self::DEFAULT_DAYS
    }

    pub fn default_limit() -> u8 {
        Self::DEFAULT_LIMIT
    }
}
