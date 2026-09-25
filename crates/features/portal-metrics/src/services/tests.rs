use std::time::Instant;

use serde_json::json;

use super::HostReader;
use crate::providers::HostMetricsProvider;
use crate::types::MetricsSettings;

#[test]
fn a_reading_describes_this_host() {
    let reader = HostReader::new();
    let reading = reader.read(&MetricsSettings::default());
    assert!(reading.memory.total_bytes > 0);
    assert!(reading.memory.used_bytes <= reading.memory.total_bytes);
    assert!(reading.uptime_seconds > 0);
    assert!(reading.cpu_percent >= 0.0 && reading.cpu_percent <= 100.0 * 64.0);
    assert!(
        !reading.disks.is_empty(),
        "a host has at least one real filesystem"
    );
    assert!(reading.disks.iter().all(|disk| disk.total_bytes > 0));
}

#[test]
fn pseudo_filesystems_are_left_out_and_mount_points_are_unique_and_sorted() {
    let reading = HostReader::new().read(&MetricsSettings::default());
    let mounts: Vec<&str> = reading
        .disks
        .iter()
        .map(|disk| disk.mount_point.as_str())
        .collect();
    let mut sorted = mounts.clone();
    sorted.sort_unstable();
    assert_eq!(mounts, sorted);
    for disk in &reading.disks {
        assert!(
            !HostReader::PSEUDO_FILE_SYSTEMS
                .contains(&disk.file_system.to_ascii_lowercase().as_str()),
            "{} is a pseudo filesystem",
            disk.file_system
        );
    }
}

#[test]
fn the_disks_setting_chooses_mount_points_in_its_own_order() {
    let reader = HostReader::new();
    let all = reader.read(&MetricsSettings::default());
    let first = all.disks[0].mount_point.clone();
    let chosen = reader.read(&MetricsSettings {
        disks: Some(vec!["/nowhere".to_string(), first.clone()]),
    });
    assert_eq!(
        chosen
            .disks
            .iter()
            .map(|disk| disk.mount_point.clone())
            .collect::<Vec<_>>(),
        vec![first]
    );
}

#[test]
fn a_reading_is_gathered_well_within_its_budget() {
    let reader = HostReader::new();
    reader.read(&MetricsSettings::default());
    let started = Instant::now();
    reader.read(&MetricsSettings::default());
    let taken = started.elapsed();
    assert!(taken.as_millis() < 500, "a reading took {taken:?}");
}

#[test]
fn the_reader_never_asks_the_host_about_processes() {
    let source = include_str!("reader.rs");
    assert!(
        !source.contains("process"),
        "the widget must not enumerate processes"
    );
    assert!(source.contains("RefreshKind::nothing()"));
}

#[test]
fn a_setting_that_is_not_a_list_of_mount_points_is_refused_by_name() {
    let provider = HostMetricsProvider::new();
    let errors = portal_feature::WidgetProvider::check(&provider, &json!({ "disks": 5 }));
    assert_eq!(errors[0].field, "disks");
    let unknown = portal_feature::WidgetProvider::check(&provider, &json!({ "disk": ["/"] }));
    assert_eq!(unknown.len(), 1);
    assert!(portal_feature::WidgetProvider::check(&provider, &json!({})).is_empty());
}
