use std::time::Duration;

use async_trait::async_trait;
use portal_feature::{FieldError, WidgetProblem, WidgetProvider};
use serde_json::Value;

use crate::clients::{Forecast, OpenMeteo};
use crate::helpers::condition_of;
use crate::types::{CurrentWeather, DailyWeather, WeatherReading, WeatherSettings};

pub struct WeatherProvider {
    client: OpenMeteo,
    timezone: String,
}

impl WeatherProvider {
    pub const KIND: &'static str = "weather";
    pub const REFRESH: Duration = Duration::from_secs(15 * 60);
    pub const SHORTEST_REFRESH: Duration = Duration::from_secs(10 * 60);

    pub fn new(endpoint: &str, timezone: &str) -> Result<WeatherProvider, String> {
        Ok(WeatherProvider {
            client: OpenMeteo::new(endpoint)?,
            timezone: timezone.to_string(),
        })
    }

    fn settings(settings: &Value) -> Result<WeatherSettings, String> {
        serde_json::from_value(settings.clone()).map_err(|error| error.to_string())
    }

    fn reading(
        settings: &WeatherSettings,
        timezone: &str,
        forecast: Forecast,
    ) -> Result<WeatherReading, WidgetProblem> {
        let current = forecast
            .current
            .ok_or_else(|| WidgetProblem::new("the forecast carried no current conditions"))?;
        let code = current.weather_code.unwrap_or(-1);
        let daily = forecast.daily.map(|block| {
            block
                .time
                .iter()
                .enumerate()
                .map(|(index, date)| {
                    let code = block.weather_code.get(index).copied().unwrap_or(-1);
                    DailyWeather {
                        date: date.clone(),
                        condition: condition_of(code).to_string(),
                        weather_code: code,
                        temperature_minimum: block
                            .temperature_2m_min
                            .get(index)
                            .copied()
                            .unwrap_or_default(),
                        temperature_maximum: block
                            .temperature_2m_max
                            .get(index)
                            .copied()
                            .unwrap_or_default(),
                        precipitation_chance: block
                            .precipitation_probability_max
                            .get(index)
                            .copied()
                            .flatten(),
                    }
                })
                .collect()
        });
        Ok(WeatherReading {
            current: CurrentWeather {
                temperature: current.temperature_2m.unwrap_or_default(),
                apparent_temperature: current.apparent_temperature.unwrap_or_default(),
                humidity_percent: current.relative_humidity_2m,
                wind_speed: current.wind_speed_10m.unwrap_or_default(),
                condition: condition_of(code).to_string(),
                weather_code: code,
                is_day: current.is_day.unwrap_or(1) == 1,
            },
            daily: daily.unwrap_or_default(),
            units: settings.units,
            timezone: forecast.timezone.unwrap_or_else(|| timezone.to_string()),
        })
    }
}

#[async_trait]
impl WidgetProvider for WeatherProvider {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn refresh(&self) -> Duration {
        Self::REFRESH.max(Self::SHORTEST_REFRESH)
    }

    fn check(&self, settings: &Value) -> Vec<FieldError> {
        let settings = match Self::settings(settings) {
            Ok(settings) => settings,
            Err(message) => return vec![FieldError::new("settings", message)],
        };
        let mut errors = Vec::new();
        if !(-90.0..=90.0).contains(&settings.latitude) {
            errors.push(FieldError::new("latitude", "must be between -90 and 90"));
        }
        if !(-180.0..=180.0).contains(&settings.longitude) {
            errors.push(FieldError::new("longitude", "must be between -180 and 180"));
        }
        if !WeatherSettings::DAYS.contains(&settings.days) {
            errors.push(FieldError::new("days", "must be between 1 and 7"));
        }
        if settings
            .timezone
            .as_ref()
            .is_some_and(|timezone| timezone.trim().is_empty())
        {
            errors.push(FieldError::new("timezone", "must not be empty"));
        }
        errors
    }

    async fn data(&self, settings: &Value) -> Result<Value, WidgetProblem> {
        let settings = Self::settings(settings).map_err(WidgetProblem::new)?;
        let timezone = settings.timezone_or(&self.timezone);
        let forecast = self.client.fetch(&settings, &timezone).await?;
        let reading = Self::reading(&settings, &timezone, forecast)?;
        serde_json::to_value(reading).map_err(|error| WidgetProblem::new(error.to_string()))
    }
}
