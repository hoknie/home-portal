use serde::{Deserialize, Serialize};

use super::{DailyWeather, Units};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentWeather {
    pub temperature: f64,
    pub apparent_temperature: f64,
    pub humidity_percent: Option<i64>,
    pub wind_speed: f64,
    pub condition: String,
    pub weather_code: i64,
    pub is_day: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherReading {
    pub current: CurrentWeather,
    pub daily: Vec<DailyWeather>,
    pub units: Units,
    pub timezone: String,
}
