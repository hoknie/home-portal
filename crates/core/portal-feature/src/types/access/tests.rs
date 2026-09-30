use super::{Action, Area, Requirement, Right, Rights};

fn names(area: Area) -> Vec<&'static str> {
    area.actions().iter().map(|action| action.name()).collect()
}

#[test]
fn every_area_accepts_exactly_the_actions_of_the_matrix() {
    let everything = vec!["read", "create", "update", "delete", "execute"];
    for area in [Area::Automations, Area::Workflows, Area::Webhooks] {
        assert_eq!(names(area), everything);
    }
    for area in [Area::Users, Area::Scripts] {
        assert_eq!(names(area), vec!["read", "create", "update", "delete"]);
    }
    for area in [
        Area::Proxy,
        Area::Dns,
        Area::Notifications,
        Area::Network,
        Area::HostPermissions,
        Area::Modules,
    ] {
        assert_eq!(names(area), vec!["read", "update"]);
    }
    assert_eq!(names(Area::Services), vec!["create", "update", "delete"]);
    for area in [Area::Layout, Area::Portal] {
        assert_eq!(names(area), vec!["update"]);
    }
    assert_eq!(names(Area::Secrets), vec!["read"]);
    assert_eq!(
        Area::from_name("host-permissions"),
        Some(Area::HostPermissions)
    );
    assert_eq!(Action::from_name("execute"), Some(Action::Execute));
}

#[test]
fn admin_allows_every_right_and_nothing_allows_none() {
    for right in Right::every() {
        assert!(Rights::admin().allows(right), "{right}");
        assert!(!Rights::none().allows(right), "{right}");
    }
}

#[test]
fn update_does_not_imply_read() {
    let rights = Rights::of([Right::new(Area::Network, Action::Update)]);
    assert!(rights.allows(Right::new(Area::Network, Action::Update)));
    assert!(!rights.allows(Right::new(Area::Network, Action::Read)));
}

#[test]
fn a_right_the_area_does_not_have_is_dropped() {
    let rights = Rights::of([Right::new(Area::Layout, Action::Delete)]);
    assert_eq!(rights, Rights::none());
}

#[test]
fn within_holds_for_a_subset_and_fails_for_a_superset() {
    let small = Rights::of([Right::new(Area::Automations, Action::Read)]);
    let large = Rights::of([
        Right::new(Area::Automations, Action::Read),
        Right::new(Area::Automations, Action::Execute),
    ]);
    assert!(small.within(&large));
    assert!(!large.within(&small));
    assert!(large.within(&Rights::admin()));
    assert!(!Rights::admin().within(&large));
    assert!(Rights::none().within(&small));
}

#[test]
fn by_area_lists_areas_and_actions_in_matrix_order() {
    let rights = Rights::of([
        Right::new(Area::Automations, Action::Execute),
        Right::new(Area::Automations, Action::Read),
        Right::new(Area::Services, Action::Update),
    ]);
    assert_eq!(
        rights.by_area(),
        vec![
            (Area::Services, vec![Action::Update]),
            (Area::Automations, vec![Action::Read, Action::Execute]),
        ]
    );
}

#[test]
fn a_requirement_is_met_by_any_of_its_rights() {
    const EITHER: &[Right] = &[
        Right::new(Area::Automations, Action::Read),
        Right::new(Area::Workflows, Action::Read),
    ];
    let workflows = Rights::of([Right::new(Area::Workflows, Action::Read)]);
    assert!(Requirement::AnyOf(EITHER).met_by(&workflows));
    assert!(!Requirement::AnyOf(EITHER).met_by(&Rights::none()));
    assert!(Requirement::Signed.met_by(&Rights::none()));
    assert!(!Requirement::Admin.met_by(&workflows));
    assert!(Requirement::Admin.met_by(&Rights::admin()));
    assert_eq!(
        Requirement::AnyOf(EITHER).describe(),
        "automations.read or workflows.read"
    );
}
