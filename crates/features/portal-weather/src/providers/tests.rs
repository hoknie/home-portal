use portal_feature::WidgetProvider;
use serde_json::json;

use super::WeatherProvider;
use crate::fakes::ForecastService;

const FORECAST: &str = r#"{
  "timezone": "Europe/Riga",
  "current": {
    "temperature_2m": 12.4,
    "apparent_temperature": 10.9,
    "relative_humidity_2m": 71,
    "wind_speed_10m": 9.3,
    "weather_code": 61,
    "is_day": 0
  },
  "daily": {
    "time": ["2026-09-22", "2026-09-23", "2026-09-24"],
    "weather_code": [61, 3, 7],
    "temperature_2m_max": [14.0, 16.5, 17.0],
    "temperature_2m_min": [8.0, 9.5, 10.0],
    "precipitation_probability_max": [80, 20, null]
  }
}"#;

fn settings() -> serde_json::Value {
    json!({ "latitude": 56.95, "longitude": 24.11, "days": 3 })
}

#[tokio::test]
async fn a_forecast_becomes_the_widget_data() {
    let upstream = ForecastService::start(FORECAST, 200).await;
    let provider = WeatherProvider::new(&upstream.endpoint(), "Europe/Moscow").unwrap();
    let data = provider.data(&settings()).await.unwrap();
    assert_eq!(data["current"]["temperature"], 12.4);
    assert_eq!(data["current"]["condition"], "rain");
    assert_eq!(data["current"]["weather_code"], 61);
    assert_eq!(data["current"]["is_day"], false);
    assert_eq!(data["current"]["humidity_percent"], 71);
    assert_eq!(data["timezone"], "Europe/Riga");
    assert_eq!(data["units"], "metric");
    let daily = data["daily"].as_array().unwrap();
    assert_eq!(daily.len(), 3);
    assert_eq!(daily[0]["date"], "2026-09-22");
    assert_eq!(daily[1]["condition"], "overcast");
    assert_eq!(daily[0]["precipitation_chance"], 80);
    assert!(daily[2]["precipitation_chance"].is_null());
}

#[tokio::test]
async fn a_code_the_portal_does_not_map_keeps_its_number() {
    let upstream = ForecastService::start(FORECAST, 200).await;
    let provider = WeatherProvider::new(&upstream.endpoint(), "Europe/Moscow").unwrap();
    let data = provider.data(&settings()).await.unwrap();
    assert_eq!(data["daily"][2]["condition"], "unknown");
    assert_eq!(data["daily"][2]["weather_code"], 7);
}

#[tokio::test]
async fn the_request_carries_only_what_the_forecast_needs() {
    let upstream = ForecastService::start(FORECAST, 200).await;
    let provider = WeatherProvider::new(&upstream.endpoint(), "Europe/Moscow").unwrap();
    provider.data(&settings()).await.unwrap();
    let request = upstream.requests().first().cloned().unwrap_or_default();
    let line = request.lines().next().unwrap_or_default().to_string();
    assert!(
        line.contains("latitude=56.95") && line.contains("longitude=24.11"),
        "{line}"
    );
    assert!(line.contains("timezone=Europe%2FMoscow"), "{line}");
    assert!(line.contains("forecast_days=3"), "{line}");
    assert!(line.contains("temperature_unit=celsius"), "{line}");
    let lower = request.to_lowercase();
    assert!(!lower.contains("cookie"), "{request}");
    assert!(!lower.contains("authorization"), "{request}");
    assert!(
        !lower.contains("home-portal"),
        "the portal does not name itself to the forecast: {request}"
    );
}

#[tokio::test]
async fn an_unhappy_forecast_becomes_a_problem_rather_than_data() {
    let upstream = ForecastService::start("nope", 503).await;
    let provider = WeatherProvider::new(&upstream.endpoint(), "Europe/Moscow").unwrap();
    let problem = provider.data(&settings()).await.unwrap_err();
    assert!(problem.to_string().contains("503"), "{problem}");
}

#[test]
fn the_settings_are_checked_field_by_field() {
    let provider =
        WeatherProvider::new("http://example.invalid/v1/forecast", "Europe/Moscow").unwrap();
    let fields: Vec<String> = provider
        .check(&json!({ "latitude": 120.0, "longitude": -200.0, "days": 9 }))
        .into_iter()
        .map(|error| error.field)
        .collect();
    assert_eq!(fields, vec!["latitude", "longitude", "days"]);
    assert!(provider.check(&settings()).is_empty());
    assert_eq!(
        provider.check(&json!({ "latitude": 1.0 }))[0].field,
        "settings"
    );
}

#[test]
fn the_forecast_is_never_asked_more_often_than_every_ten_minutes() {
    let provider =
        WeatherProvider::new("http://example.invalid/v1/forecast", "Europe/Moscow").unwrap();
    assert!(provider.refresh() >= WeatherProvider::SHORTEST_REFRESH);
    assert_eq!(provider.refresh(), WeatherProvider::REFRESH);
}
