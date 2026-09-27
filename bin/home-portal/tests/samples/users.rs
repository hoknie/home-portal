use portal_auth::{UserResponse, UsersResponse};

use crate::check;

#[test]
fn the_users_sample_matches_its_serializer() {
    let users = UsersResponse {
        users: vec![
            UserResponse {
                name: "admin".into(),
                you: true,
            },
            UserResponse {
                name: "anna".into(),
                you: false,
            },
        ],
        editable: true,
    };
    check("users", serde_json::to_value(users).unwrap());
}
