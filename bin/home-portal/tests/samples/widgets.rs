use super::*;

fn envelope(data: Value, refresh_seconds: u64) -> Value {
    serde_json::to_value(WidgetData {
        data,
        fetched_at: datetime!(2026-09-22 10:00 UTC),
        stale: false,
        problem: None,
        refresh_seconds,
    })
    .unwrap()
}

#[test]
fn the_host_metrics_sample_matches_its_serializer() {
    let reading = HostReading {
        hostname: Some("box".into()),
        cpu_percent: 17.5,
        load_average: Some([0.42, 0.51, 0.63]),
        memory: Usage {
            used_bytes: 6_871_947_674,
            total_bytes: 17_179_869_184,
        },
        swap: Usage {
            used_bytes: 0,
            total_bytes: 2_147_483_648,
        },
        disks: vec![
            DiskReading {
                mount_point: "/".into(),
                file_system: "apfs".into(),
                total_bytes: 494_384_795_648,
                available_bytes: 128_849_018_880,
            },
            DiskReading {
                mount_point: "/media".into(),
                file_system: "ext4".into(),
                total_bytes: 3_998_639_390_720,
                available_bytes: 419_430_400_000,
            },
        ],
        uptime_seconds: 93_600,
    };
    check(
        "widget-host-metrics",
        envelope(serde_json::to_value(reading).unwrap(), 10),
    );
}

#[test]
fn the_weather_sample_matches_its_serializer() {
    let reading = WeatherReading {
        current: CurrentWeather {
            temperature: 12.4,
            apparent_temperature: 10.9,
            humidity_percent: Some(78),
            wind_speed: 14.8,
            condition: "partly-cloudy".into(),
            weather_code: 2,
            is_day: true,
        },
        daily: vec![
            DailyWeather {
                date: "2026-09-22".into(),
                condition: "rain".into(),
                weather_code: 61,
                temperature_minimum: 8.1,
                temperature_maximum: 14.2,
                precipitation_chance: Some(70),
            },
            DailyWeather {
                date: "2026-09-23".into(),
                condition: "clear".into(),
                weather_code: 0,
                temperature_minimum: 7.4,
                temperature_maximum: 16.0,
                precipitation_chance: Some(5),
            },
        ],
        units: Units::Metric,
        timezone: "+03:00".into(),
    };
    check(
        "widget-weather",
        envelope(serde_json::to_value(reading).unwrap(), 900),
    );
}

#[test]
fn the_calendar_sample_matches_its_serializer() {
    let events = vec![
        CalendarEvent {
            summary: "Pick up a parcel".into(),
            start: datetime!(2026-09-22 09:00 UTC),
            end: Some(datetime!(2026-09-22 09:30 UTC)),
            all_day: false,
            location: Some("Post office".into()),
            calendar: Some("Home".into()),
            repeats: Repeats::Never,
        },
        CalendarEvent {
            summary: "Birthday".into(),
            start: datetime!(2026-09-23 00:00 UTC),
            end: None,
            all_day: true,
            location: None,
            calendar: Some("Home".into()),
            repeats: Repeats::Unsupported,
        },
    ];
    check(
        "widget-calendar",
        envelope(json!({ "events": events }), 1800),
    );
}

#[tokio::test]
async fn the_widget_kinds_sample_names_every_registered_provider() {
    let directory = tempfile::tempdir().unwrap();
    let path = support::with_extra(&directory, "secret", "");
    let registry = registered(&support::wiring_for(&path)).unwrap();
    let mut conditions: Vec<&str> = CONDITIONS
        .iter()
        .map(|(_, condition)| *condition)
        .chain([UNKNOWN_CONDITION])
        .collect();
    conditions.sort_unstable();
    conditions.dedup();
    check(
        "widget-kinds",
        json!({ "kinds": registry.widgets.kinds(), "conditions": conditions }),
    );
}
