use axum::body::Body;
use axum::http::{Method, Request};
use portal_feature::{Action, ApiError, Area, Principal, Right, Rights, Rule};

use super::require_right::{NO_RULE, refusal};
use crate::types::RuleBook;

const RUN: &[Right] = &[Right::new(Area::Automations, Action::Execute)];
const EITHER: &[Right] = &[
    Right::new(Area::Automations, Action::Read),
    Right::new(Area::Workflows, Action::Read),
];

fn book() -> RuleBook {
    RuleBook::of([
        Rule::needs(Method::POST, "/api/run", RUN),
        Rule::needs(Method::GET, "/api/runs", EITHER),
        Rule::admin(Method::POST, "/api/groups"),
        Rule::signed(Method::GET, "/api/services"),
    ])
}

fn request(method: Method, path: &str, rights: Option<Rights>) -> Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    if let Some(rights) = rights {
        request
            .extensions_mut()
            .insert(Principal::member("anna", None, rights));
    }
    request
}

fn refused(method: Method, path: &str, rights: Rights) -> Option<String> {
    refusal(&book(), &request(method, path, Some(rights))).map(|error| match error {
        ApiError::Forbidden(message) => message,
        other => panic!("not a 403: {other:?}"),
    })
}

fn only(area: Area, action: Action) -> Rights {
    Rights::of([Right::new(area, action)])
}

#[test]
fn a_missing_right_is_refused_naming_it() {
    assert_eq!(
        refused(
            Method::POST,
            "/api/run",
            only(Area::Automations, Action::Read)
        ),
        Some("needs automations.execute".to_string())
    );
    assert_eq!(
        refused(
            Method::POST,
            "/api/run",
            only(Area::Automations, Action::Execute)
        ),
        None
    );
}

#[test]
fn either_right_of_any_of_is_enough() {
    assert_eq!(
        refused(
            Method::GET,
            "/api/runs",
            only(Area::Workflows, Action::Read)
        ),
        None
    );
    assert!(refused(Method::GET, "/api/runs", Rights::none()).is_some());
}

#[test]
fn admin_routes_admit_only_admin() {
    assert!(
        refused(
            Method::POST,
            "/api/groups",
            only(Area::Users, Action::Update)
        )
        .is_some()
    );
    assert_eq!(refused(Method::POST, "/api/groups", Rights::admin()), None);
}

#[test]
fn a_signed_route_admits_anyone_with_a_session() {
    assert_eq!(refused(Method::GET, "/api/services", Rights::none()), None);
}

#[test]
fn a_route_without_a_rule_is_closed() {
    assert_eq!(
        refused(Method::DELETE, "/api/services", Rights::admin()),
        Some(NO_RULE.to_string())
    );
}

#[test]
fn without_a_principal_nothing_passes() {
    assert!(refusal(&book(), &request(Method::GET, "/api/services", None)).is_some());
}
