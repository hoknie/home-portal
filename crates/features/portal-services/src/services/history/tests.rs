use portal_model::{Diagnosis, ProbeOutcome, ServiceState};

use super::ServiceHistory;
use crate::types::HistoryRange;

const START: i64 = 1_790_000_000;

fn up(latency: u64) -> ProbeOutcome {
    ProbeOutcome::answered(ServiceState::Up, latency)
}

fn down() -> ProbeOutcome {
    ProbeOutcome::failed(ServiceState::Down, None, "connection refused".into())
        .because(Diagnosis::Refused)
}

fn probed_every(history: &mut ServiceHistory, from: i64, until: i64, every: i64) {
    let mut at = from;
    while at < until {
        history.record(at, &up(10));
        at += every;
    }
}

#[test]
fn forty_days_of_probes_keep_a_day_of_samples_thirty_days_of_hours_and_nothing_older() {
    let mut history = ServiceHistory::default();
    let end = START + 40 * 86_400;
    let mut at = START;
    while at < end {
        let outcome = if (at - START) % 86_400 == 0 {
            down()
        } else {
            up(10)
        };
        history.record(at, &outcome);
        at += 30;
    }
    let last = end - 30;
    assert_eq!(history.samples.len(), 2_880);
    assert!(history.samples.front().unwrap().at >= last - 86_400);
    assert!(history.buckets.len() <= 721 && history.buckets.len() >= 720);
    assert!(history.buckets.front().unwrap().hour + 3600 > last - 30 * 86_400);
    assert!(
        history
            .transitions
            .iter()
            .all(|transition| transition.at >= last - 30 * 86_400)
    );
    assert!(!history.transitions.is_empty());
}

#[test]
fn an_hour_bucket_keeps_minimum_average_maximum_and_covered_time() {
    let mut history = ServiceHistory::default();
    let hour = START - START.rem_euclid(3600);
    for (offset, latency) in [(0, 10), (30, 20), (60, 60)] {
        history.record(hour + offset, &up(latency));
    }
    history.record(hour + 90, &down());
    let bucket = history.buckets.back().unwrap();
    assert_eq!(bucket.minimum, Some(10));
    assert_eq!(bucket.maximum, Some(60));
    assert_eq!(bucket.average(), Some(30));
    assert_eq!(bucket.covered_seconds, 90);
    assert_eq!(bucket.answered_seconds, 60);
    assert_eq!(bucket.up, 3);
    assert_eq!(bucket.down, 1);
    assert_eq!(bucket.worst(), ServiceState::Down);
}

#[test]
fn time_without_probes_is_not_downtime() {
    let mut history = ServiceHistory::default();
    let now = START + 86_400;
    probed_every(&mut history, START, START + 43_200, 30);
    probed_every(&mut history, now - 60, now + 1, 30);
    let uptime = history.uptime(HistoryRange::Day, now);
    assert_eq!(uptime.ratio, Some(1.0));
    let hours = uptime.covered_seconds as f64 / 3600.0;
    assert!((11.9..12.2).contains(&hours), "{hours}");
}

#[test]
fn nothing_probed_means_no_uptime_figure() {
    let history = ServiceHistory::default();
    assert_eq!(history.uptime(HistoryRange::Week, START).ratio, None);
}

#[test]
fn a_state_change_is_recorded_with_the_error_that_started_it() {
    let mut history = ServiceHistory::default();
    history.record(START, &up(5));
    history.record(START + 30, &up(5));
    history.record(START + 60, &down());
    history.record(START + 90, &up(5));
    let changes: Vec<_> = history
        .transitions
        .iter()
        .map(|transition| (transition.from, transition.to, transition.error.clone()))
        .collect();
    assert_eq!(
        changes,
        vec![
            (ServiceState::Unknown, ServiceState::Up, None),
            (
                ServiceState::Up,
                ServiceState::Down,
                Some("connection refused".to_string())
            ),
            (ServiceState::Down, ServiceState::Up, None),
        ]
    );
}

#[test]
fn probes_closer_than_five_seconds_are_thinned_into_one_sample() {
    let mut history = ServiceHistory::default();
    history.record(START, &up(5));
    history.record(START + 30, &up(7));
    history.record(START + 32, &up(9));
    history.record(START + 34, &up(11));
    assert_eq!(history.samples.len(), 2);
    assert_eq!(history.samples.back().unwrap().latency, Some(11));
    assert_eq!(history.samples.back().unwrap().covered, 34);
    assert_eq!(history.buckets.back().unwrap().latency_count, 4);
}

#[test]
fn a_day_view_lists_single_outcomes_and_a_week_view_lists_hours() {
    let mut history = ServiceHistory::default();
    probed_every(&mut history, START, START + 7200, 60);
    let now = START + 7200;
    let day = history.view(HistoryRange::Day, now);
    assert_eq!(day.points.len(), 120);
    assert_eq!(day.uptime.len(), 3);
    let week = history.view(HistoryRange::Week, now);
    assert!(week.points.len() <= 3 && week.points.len() >= 2);
    assert!(week.points.iter().all(|point| point.average == Some(10)));
}

#[test]
fn a_history_survives_a_round_trip_through_json_without_its_dirty_mark() {
    let mut history = ServiceHistory::default();
    history.record(START, &up(5));
    history.record(START + 30, &down());
    let text = serde_json::to_string(&history).unwrap();
    let restored: ServiceHistory = serde_json::from_str(&text).unwrap();
    assert!(!restored.dirty);
    assert_eq!(restored.samples, history.samples);
    assert_eq!(restored.transitions, history.transitions);
}
