use std::net::IpAddr;
use std::sync::Arc;

use portal_feature::ApiError;

use super::super::PasswordChecks;
use crate::helpers::hash_password;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_burst_from_one_address_checks_at_most_five_passwords() {
    let checks = Arc::new(PasswordChecks::default());
    let hash = hash_password("the right one").unwrap();
    let client: IpAddr = "203.0.113.7".parse().unwrap();
    let mut attempts = Vec::new();
    for _ in 0..50 {
        let checks = checks.clone();
        let hash = hash.clone();
        attempts.push(tokio::spawn(async move {
            checks
                .verify(client, "a wrong one".to_string(), Some(hash))
                .await
        }));
    }
    let mut checked = 0;
    let mut refused = 0;
    for attempt in attempts {
        match attempt.await.unwrap() {
            Ok(false) => checked += 1,
            Err(ApiError::TooManyRequests { .. }) => refused += 1,
            other => panic!("{other:?}"),
        }
    }
    assert!(checked <= 5, "{checked} passwords were checked");
    assert_eq!(checked + refused, 50);
}
