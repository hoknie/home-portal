use std::net::{IpAddr, Ipv4Addr};

use time::Duration;
use time::macros::datetime;
use toml_edit::DocumentMut;

use super::{SessionStore, Throttle, validate_users};
use crate::helpers::hash_password;

const CLIENT: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20));

fn document(text: &str) -> DocumentMut {
    text.parse().unwrap()
}

#[test]
fn a_configuration_without_users_is_refused_with_the_command_to_run() {
    let errors = validate_users(&document("[network]\nport = 8080\n"));
    assert_eq!(errors[0].field, "users");
    assert!(errors[0].message.contains("home-portal password-hash"));
}

#[test]
fn duplicate_names_and_foreign_hashes_are_refused_by_entry() {
    let hash = hash_password("secret").unwrap();
    let text = format!(
        "[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n\n[[users]]\nname = \"admin\"\npassword_hash = \"plain\"\n"
    );
    let fields: Vec<String> = validate_users(&document(&text))
        .into_iter()
        .map(|error| error.field)
        .collect();
    assert_eq!(fields, vec!["users[1].name", "users[1].password_hash"]);
}

#[test]
fn a_valid_user_passes() {
    let hash = hash_password("secret").unwrap();
    let text = format!("[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n");
    assert!(validate_users(&document(&text)).is_empty());
}

#[test]
fn a_session_slides_with_use_and_expires_after_seven_idle_days() {
    let store = SessionStore::default();
    let start = datetime!(2026-09-22 10:00 UTC);
    let token = store.create("admin", start);
    let later = start + Duration::days(6);
    assert_eq!(
        store.admit(&token, later, |_| true).as_deref(),
        Some("admin")
    );
    assert!(
        store
            .admit(&token, later + Duration::days(6), |_| true)
            .is_some()
    );
    assert!(
        store
            .admit(
                &token,
                later + Duration::days(6) + Duration::days(8),
                |_| true
            )
            .is_none()
    );
}

#[test]
fn a_session_stops_being_valid_when_its_user_is_removed() {
    let store = SessionStore::default();
    let now = datetime!(2026-09-22 10:00 UTC);
    let token = store.create("admin", now);
    assert!(store.admit(&token, now, |name| name != "admin").is_none());
    assert!(store.admit(&token, now, |_| true).is_none());
}

#[test]
fn an_ended_session_is_gone_and_expired_ones_are_pruned() {
    let store = SessionStore::default();
    let now = datetime!(2026-09-22 10:00 UTC);
    let ended = store.create("admin", now);
    store.end(&ended);
    assert!(store.admit(&ended, now, |_| true).is_none());
    store.create("admin", now);
    assert_eq!(store.prune(now + Duration::days(8)), 1);
}

#[test]
fn the_sixth_attempt_after_five_failures_is_refused_for_a_minute() {
    let throttle = Throttle::default();
    let now = datetime!(2026-09-22 10:00 UTC);
    for _ in 0..5 {
        assert!(throttle.check(CLIENT, now).is_ok());
        throttle.fail(CLIENT, now);
    }
    assert_eq!(throttle.check(CLIENT, now), Err(60));
    assert_eq!(throttle.check(CLIENT, now + Duration::seconds(59)), Err(1));
    assert!(throttle.check(CLIENT, now + Duration::seconds(61)).is_ok());
}

#[test]
fn a_success_resets_the_failures() {
    let throttle = Throttle::default();
    let now = datetime!(2026-09-22 10:00 UTC);
    for _ in 0..4 {
        throttle.fail(CLIENT, now);
    }
    throttle.succeed(CLIENT);
    throttle.fail(CLIENT, now);
    assert!(throttle.check(CLIENT, now).is_ok());
}

#[test]
fn failures_older_than_the_window_do_not_count() {
    let throttle = Throttle::default();
    let now = datetime!(2026-09-22 10:00 UTC);
    for _ in 0..4 {
        throttle.fail(CLIENT, now);
    }
    throttle.fail(CLIENT, now + Duration::minutes(16));
    assert!(throttle.check(CLIENT, now + Duration::minutes(16)).is_ok());
}

#[test]
fn the_throttle_forgets_the_oldest_client_past_its_capacity() {
    let throttle = Throttle::default();
    let now = datetime!(2026-09-22 10:00 UTC);
    for index in 0..=Throttle::CAPACITY as u32 {
        let address = IpAddr::from(index.to_be_bytes());
        throttle.fail(address, now + Duration::seconds(i64::from(index)));
    }
    assert_eq!(throttle.tracked(), Throttle::CAPACITY);
}

fn file_in(directory: &tempfile::TempDir) -> crate::repositories::SessionFile {
    crate::repositories::SessionFile::beside(&directory.path().join("home-portal.toml"))
}

#[test]
fn a_session_survives_a_restart() {
    let directory = tempfile::tempdir().unwrap();
    let now = time::macros::datetime!(2026-09-24 10:00 UTC);
    let token = SessionStore::kept_in(file_in(&directory), now).create("admin", now);
    let restarted = SessionStore::kept_in(file_in(&directory), now);
    assert_eq!(
        restarted.admit(&token, now, |_| true).as_deref(),
        Some("admin")
    );
}

#[test]
fn the_sessions_file_is_private_and_holds_no_token() {
    let directory = tempfile::tempdir().unwrap();
    let now = time::macros::datetime!(2026-09-24 10:00 UTC);
    let token = SessionStore::kept_in(file_in(&directory), now).create("admin", now);
    let path = directory.path().join("sessions.json");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(!text.contains(&token));
    assert!(text.contains("admin"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}

#[test]
fn signing_out_and_expiry_are_not_brought_back_by_a_restart() {
    let directory = tempfile::tempdir().unwrap();
    let now = time::macros::datetime!(2026-09-24 10:00 UTC);
    let store = SessionStore::kept_in(file_in(&directory), now);
    let ended = store.create("admin", now);
    let expiring = store.create("admin", now);
    store.end(&ended);
    let later = now + SessionStore::LIFETIME + time::Duration::seconds(1);
    let restarted = SessionStore::kept_in(file_in(&directory), later);
    assert!(restarted.admit(&ended, later, |_| true).is_none());
    assert!(restarted.admit(&expiring, later, |_| true).is_none());
}

#[test]
fn an_extension_is_kept_after_a_flush() {
    let directory = tempfile::tempdir().unwrap();
    let start = time::macros::datetime!(2026-09-24 10:00 UTC);
    let store = SessionStore::kept_in(file_in(&directory), start);
    let token = store.create("admin", start);
    let used = start + time::Duration::days(6);
    store.admit(&token, used, |_| true);
    store.flush();
    let late = start + time::Duration::days(10);
    let restarted = SessionStore::kept_in(file_in(&directory), late);
    assert_eq!(
        restarted.admit(&token, late, |_| true).as_deref(),
        Some("admin")
    );
}

#[test]
fn an_unreadable_sessions_file_is_set_aside() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("sessions.json"), "{not json").unwrap();
    let now = time::macros::datetime!(2026-09-24 10:00 UTC);
    let store = SessionStore::kept_in(file_in(&directory), now);
    assert!(store.admit("anything", now, |_| true).is_none());
    assert!(directory.path().join("sessions.json.broken").exists());
}
