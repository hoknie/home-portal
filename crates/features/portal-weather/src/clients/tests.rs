use super::OpenMeteo;
use crate::types::{Units, WeatherSettings};

#[test]
fn the_target_names_the_fields_the_widget_shows() {
    let client = OpenMeteo::new(OpenMeteo::ENDPOINT).unwrap();
    let settings = WeatherSettings {
        latitude: 1.5,
        longitude: 2.5,
        timezone: None,
        units: Units::Imperial,
        days: 5,
    };
    let target = client.target(&settings, "Europe/Riga");
    let query = target.query().unwrap();
    assert!(query.contains("current=temperature_2m"), "{query}");
    assert!(query.contains("daily=weather_code"), "{query}");
    assert!(query.contains("temperature_unit=fahrenheit"), "{query}");
    assert!(query.contains("wind_speed_unit=mph"), "{query}");
    assert_eq!(target.host_str(), Some("api.open-meteo.com"));
}
