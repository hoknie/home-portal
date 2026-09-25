use super::*;

#[test]
fn the_history_sample_matches_its_serializer() {
    let start = datetime!(2026-09-22 10:00 UTC).unix_timestamp();
    let mut history = ServiceHistory::default();
    for step in 0..6 {
        let at = start + step * 30;
        let outcome = if step == 3 {
            ProbeOutcome::failed(ServiceState::Down, None, "connection refused".into())
                .because(Diagnosis::Refused)
        } else {
            ProbeOutcome::answered(ServiceState::Up, 40 + u64::try_from(step).unwrap())
        };
        history.record(at, &outcome);
    }
    let view = history.view(HistoryRange::Day, start + 180);
    check(
        "history",
        serde_json::to_value(HistoryResponse::of(&view)).unwrap(),
    );
}
