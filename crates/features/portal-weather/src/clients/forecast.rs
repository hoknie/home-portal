use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Forecast {
    pub timezone: Option<String>,
    pub current: Option<CurrentBlock>,
    pub daily: Option<DailyBlock>,
}

#[derive(Debug, Deserialize)]
pub struct CurrentBlock {
    pub temperature_2m: Option<f64>,
    pub apparent_temperature: Option<f64>,
    pub relative_humidity_2m: Option<i64>,
    pub wind_speed_10m: Option<f64>,
    pub weather_code: Option<i64>,
    pub is_day: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DailyBlock {
    #[serde(default)]
    pub time: Vec<String>,
    #[serde(default)]
    pub weather_code: Vec<i64>,
    #[serde(default)]
    pub temperature_2m_max: Vec<f64>,
    #[serde(default)]
    pub temperature_2m_min: Vec<f64>,
    #[serde(default)]
    pub precipitation_probability_max: Vec<Option<i64>>,
}
