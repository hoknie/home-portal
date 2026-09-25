use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyWeather {
    pub date: String,
    pub condition: String,
    pub weather_code: i64,
    pub temperature_minimum: f64,
    pub temperature_maximum: f64,
    pub precipitation_chance: Option<i64>,
}
