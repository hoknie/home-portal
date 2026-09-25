use portal_feature::{EventName, PortalEvent, Visitor};
use time::OffsetDateTime;

use super::support::{automation, automations, entry, status_change};
use crate::services::matching;

fn signed_in(environment: &str) -> PortalEvent {
    PortalEvent::visited(
        EventName::UserSignedIn,
        &Visitor {
            user: "admin".into(),
            address: "10.0.0.2".into(),
            environment: environment.into(),
            reason: None,
        },
        OffsetDateTime::UNIX_EPOCH,
    )
}

#[test]
fn only_the_failure_of_the_named_service_matches() {
    let list = automations(&entry(
        "restart-nas",
        "{ event = \"service.status-changed\", services = [\"nas\"], to = [\"down\", \"unreadable\"] }",
        "",
    ));
    assert_eq!(
        matching(&list, &status_change("nas", "up", "down")).len(),
        1
    );
    assert!(matching(&list, &status_change("plex", "up", "down")).is_empty());
    assert!(matching(&list, &status_change("nas", "down", "up")).is_empty());
}

#[test]
fn leaving_unknown_matches_only_with_the_flag() {
    let plain = automation(
        "a",
        "{ event = \"service.status-changed\", to = [\"up\"] }",
        "",
    );
    let flagged = automation(
        "b",
        "{ event = \"service.status-changed\", to = [\"up\"], from_unknown = true }",
        "",
    );
    let list = [plain, flagged];
    let found = matching(&list, &status_change("nas", "unknown", "up"));
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, "b");
}

#[test]
fn only_a_sign_in_from_the_internet_matches() {
    let list = automations(&entry(
        "audit",
        "{ event = \"user.signed-in\", environments = [\"internet\"] }",
        "",
    ));
    assert!(matching(&list, &signed_in("home")).is_empty());
    assert_eq!(matching(&list, &signed_in("internet")).len(), 1);
}

#[test]
fn a_disabled_automation_never_matches() {
    let list = automations(&entry(
        "a",
        "{ event = \"portal.started\" }",
        "enabled = false",
    ));
    let event = PortalEvent::portal(EventName::PortalStarted, "", OffsetDateTime::UNIX_EPOCH);
    assert!(matching(&list, &event).is_empty());
}

#[test]
fn a_targeted_event_matches_only_its_target_and_a_schedule_is_always_targeted() {
    let text = entry("a", "{ event = \"schedule\", cron = \"@daily\" }", "")
        + &entry("b", "{ event = \"schedule\", cron = \"@daily\" }", "");
    let list = automations(&text);
    let event = PortalEvent::of(EventName::Schedule, OffsetDateTime::UNIX_EPOCH, &[]);
    assert!(matching(&list, &event).is_empty());
    let aimed = matching(&list, &event.aimed_at("b"));
    assert_eq!(aimed.len(), 1);
    assert_eq!(aimed[0].id, "b");
}

#[test]
fn a_manual_automation_never_matches_an_event() {
    let list = automations(&entry("a", "{ event = \"manual\" }", ""));
    let event = PortalEvent::of(EventName::Manual, OffsetDateTime::UNIX_EPOCH, &[]);
    assert!(matching(&list, &event).is_empty());
}
