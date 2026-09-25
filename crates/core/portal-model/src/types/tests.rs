use time::macros::datetime;

use super::{
    Diagnosis, Environment, EnvironmentError, Environments, ProbeOutcome, ServiceId,
    ServiceIdError, ServiceState, ServiceStatus,
};

#[test]
fn a_service_id_is_lower_case_letters_digits_and_hyphens_starting_with_a_letter() {
    assert!(ServiceId::parse("jellyfin").is_ok());
    assert!(ServiceId::parse("home-assistant-2").is_ok());
    assert_eq!(ServiceId::parse(""), Err(ServiceIdError::Empty));
    assert_eq!(
        ServiceId::parse("2fast"),
        Err(ServiceIdError::MustStartWithLetter)
    );
    assert_eq!(
        ServiceId::parse("Bad Id"),
        Err(ServiceIdError::MustStartWithLetter)
    );
    assert_eq!(
        ServiceId::parse("bad id"),
        Err(ServiceIdError::InvalidCharacter(' '))
    );
}

#[test]
fn a_service_id_may_be_sixty_three_characters_and_not_one_more() {
    let longest = format!("a{}", "b".repeat(62));
    assert!(ServiceId::parse(&longest).is_ok());
    assert_eq!(
        ServiceId::parse(&format!("{longest}c")),
        Err(ServiceIdError::TooLong)
    );
}

#[test]
fn since_moves_only_when_the_state_changes() {
    let start = datetime!(2026-09-22 10:00 UTC);
    let first = ServiceStatus::unknown(start).after(
        ProbeOutcome::answered(ServiceState::Up, 12),
        datetime!(2026-09-22 10:01 UTC),
    );
    assert_eq!(first.since, datetime!(2026-09-22 10:01 UTC));
    let second = first.after(
        ProbeOutcome::answered(ServiceState::Up, 15),
        datetime!(2026-09-22 10:02 UTC),
    );
    assert_eq!(second.since, datetime!(2026-09-22 10:01 UTC));
    assert_eq!(second.checked_at, Some(datetime!(2026-09-22 10:02 UTC)));
}

#[test]
fn the_last_success_is_kept_while_the_service_is_down() {
    let up = ServiceStatus::unknown(datetime!(2026-09-22 10:00 UTC)).after(
        ProbeOutcome::answered(ServiceState::Up, 9),
        datetime!(2026-09-22 10:01 UTC),
    );
    let down = up.after(
        ProbeOutcome::failed(ServiceState::Down, None, "connection refused".into()),
        datetime!(2026-09-22 10:02 UTC),
    );
    assert_eq!(down.state, ServiceState::Down);
    assert_eq!(down.last_ok_at, Some(datetime!(2026-09-22 10:01 UTC)));
    assert_eq!(down.last_error.as_deref(), Some("connection refused"));
    assert_eq!(down.since, datetime!(2026-09-22 10:02 UTC));
}

#[test]
fn a_status_serializes_with_snake_case_states_and_rfc3339_times() {
    let status = ServiceStatus::unknown(datetime!(2026-09-22 10:00 UTC));
    let json = serde_json::to_value(&status).unwrap();
    assert_eq!(json["state"], "unknown");
    assert_eq!(json["since"], "2026-09-22T10:00:00Z");
    assert!(json["checked_at"].is_null());
}

#[test]
fn an_environment_name_is_lower_case_letters_digits_and_hyphens() {
    assert_eq!(Environment::parse("local").unwrap().as_str(), "local");
    assert!(Environment::parse("home-lan").is_ok());
    assert_eq!(Environment::parse(""), Err(EnvironmentError::Empty));
    assert_eq!(
        Environment::parse("Local"),
        Err(EnvironmentError::MustStartWithLetter)
    );
    assert_eq!(
        Environment::parse("home lan"),
        Err(EnvironmentError::InvalidCharacter(' '))
    );
}

#[test]
fn the_internet_environment_knows_itself() {
    assert!(Environment::internet().is_internet());
    assert!(!Environment::parse("local").unwrap().is_internet());
    assert_eq!(Environment::internet().to_string(), "internet");
}

#[test]
fn every_diagnosis_serializes_as_its_kebab_case_code() {
    for diagnosis in Diagnosis::ALL {
        let json = serde_json::to_value(diagnosis).unwrap();
        assert_eq!(json, diagnosis.code());
    }
    assert_eq!(
        serde_json::to_value(Diagnosis::LocalNetworkDenied).unwrap(),
        "local-network-denied"
    );
}

#[test]
fn a_diagnosis_code_from_a_newer_portal_reads_as_unknown() {
    let parsed: Diagnosis = serde_json::from_str("\"cosmic-rays\"").unwrap();
    assert_eq!(parsed, Diagnosis::Unknown);
}

#[test]
fn a_success_clears_the_diagnosis_of_the_previous_failure() {
    let down = ServiceStatus::unknown(datetime!(2026-09-22 10:00 UTC)).after(
        ProbeOutcome::failed(ServiceState::Down, None, "refused".into())
            .because(Diagnosis::Refused),
        datetime!(2026-09-22 10:01 UTC),
    );
    assert_eq!(down.diagnosis, Some(Diagnosis::Refused));
    let up = down.after(
        ProbeOutcome::answered(ServiceState::Up, 9),
        datetime!(2026-09-22 10:02 UTC),
    );
    assert_eq!(up.diagnosis, None);
    assert!(serde_json::to_value(&up).unwrap()["diagnosis"].is_null());
}

#[test]
fn a_state_name_is_its_wire_value() {
    for state in ServiceState::ALL {
        assert_eq!(serde_json::to_value(state).unwrap(), state.name());
    }
}

fn home_and_vpn() -> Environments {
    Environments::new(vec![
        (
            Environment::parse("local").unwrap(),
            vec!["192.168.0.0/16".parse().unwrap()],
        ),
        (
            Environment::parse("vpn").unwrap(),
            vec!["10.8.0.0/24".parse().unwrap()],
        ),
    ])
}

#[test]
fn a_visitor_inside_may_look_from_another_known_environment() {
    let local = Environment::parse("local").unwrap();
    let environments = home_and_vpn();
    assert_eq!(
        environments.effective(&local, Some("internet")).as_str(),
        "internet"
    );
    assert_eq!(environments.effective(&local, Some("vpn")).as_str(), "vpn");
    assert_eq!(environments.effective(&local, None).as_str(), "local");
}

#[test]
fn a_choice_naming_nothing_known_is_ignored() {
    let local = Environment::parse("local").unwrap();
    let environments = home_and_vpn();
    assert_eq!(
        environments.effective(&local, Some("office")).as_str(),
        "local"
    );
    assert_eq!(
        environments.effective(&local, Some("Not Valid")).as_str(),
        "local"
    );
}

#[test]
fn a_visitor_from_the_internet_cannot_choose() {
    let environments = home_and_vpn();
    assert_eq!(
        environments
            .effective(&Environment::internet(), Some("local"))
            .as_str(),
        "internet"
    );
}
