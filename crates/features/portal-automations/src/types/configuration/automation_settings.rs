use jiff::tz::TimeZone;
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AutomationSettings {
    #[serde(default)]
    pub timezone: Option<String>,
}

impl AutomationSettings {
    pub fn zone(&self) -> Result<TimeZone, String> {
        match self.timezone.as_deref() {
            None | Some("") => Ok(TimeZone::system()),
            Some(name) => TimeZone::get(name)
                .map_err(|_| format!("names {name}, which is not a known time zone")),
        }
    }
}
