pub const UNKNOWN_CONDITION: &str = "unknown";

pub const CONDITIONS: [(i64, &str); 28] = [
    (0, "clear"),
    (1, "mostly-clear"),
    (2, "partly-cloudy"),
    (3, "overcast"),
    (45, "fog"),
    (48, "rime-fog"),
    (51, "drizzle"),
    (53, "drizzle"),
    (55, "drizzle"),
    (56, "freezing-drizzle"),
    (57, "freezing-drizzle"),
    (61, "rain"),
    (63, "rain"),
    (65, "heavy-rain"),
    (66, "freezing-rain"),
    (67, "freezing-rain"),
    (71, "snow"),
    (73, "snow"),
    (75, "heavy-snow"),
    (77, "snow-grains"),
    (80, "showers"),
    (81, "showers"),
    (82, "heavy-showers"),
    (85, "snow-showers"),
    (86, "snow-showers"),
    (95, "thunderstorm"),
    (96, "thunderstorm-hail"),
    (99, "thunderstorm-hail"),
];

pub fn condition_of(code: i64) -> &'static str {
    CONDITIONS
        .iter()
        .find(|(known, _)| *known == code)
        .map(|(_, condition)| *condition)
        .unwrap_or(UNKNOWN_CONDITION)
}
