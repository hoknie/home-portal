use std::time::Duration;

use portal_feature::WidgetProblem;
use reqwest::Client;
use url::Url;

use super::Forecast;
use crate::types::WeatherSettings;

pub struct OpenMeteo {
    client: Client,
    endpoint: Url,
}

impl OpenMeteo {
    pub const ENDPOINT: &'static str = "https://api.open-meteo.com/v1/forecast";
    pub const TIMEOUT: Duration = Duration::from_secs(10);
    pub const CURRENT_FIELDS: &'static str = "temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,weather_code,is_day";
    pub const DAILY_FIELDS: &'static str =
        "weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max";

    pub fn new(endpoint: &str) -> Result<OpenMeteo, String> {
        let client = Client::builder()
            .timeout(Self::TIMEOUT)
            .build()
            .map_err(|error| error.to_string())?;
        let endpoint = Url::parse(endpoint).map_err(|error| error.to_string())?;
        Ok(OpenMeteo { client, endpoint })
    }

    pub fn target(&self, settings: &WeatherSettings, timezone: &str) -> Url {
        let mut target = self.endpoint.clone();
        target
            .query_pairs_mut()
            .append_pair("latitude", &settings.latitude.to_string())
            .append_pair("longitude", &settings.longitude.to_string())
            .append_pair("current", Self::CURRENT_FIELDS)
            .append_pair("daily", Self::DAILY_FIELDS)
            .append_pair("timezone", timezone)
            .append_pair("forecast_days", &settings.days.to_string())
            .append_pair("temperature_unit", settings.units.temperature())
            .append_pair("wind_speed_unit", settings.units.wind());
        target
    }

    pub async fn fetch(
        &self,
        settings: &WeatherSettings,
        timezone: &str,
    ) -> Result<Forecast, WidgetProblem> {
        let response = self
            .client
            .get(self.target(settings, timezone))
            .send()
            .await
            .map_err(|error| {
                WidgetProblem::new(format!("the forecast could not be fetched: {error}"))
            })?;
        if !response.status().is_success() {
            return Err(WidgetProblem::new(format!(
                "the forecast answered {}",
                response.status()
            )));
        }
        response
            .json()
            .await
            .map_err(|error| WidgetProblem::new(format!("the forecast could not be read: {error}")))
    }
}
