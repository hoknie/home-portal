use std::time::Duration;

use super::{next_wait, probe_target};

#[test]
fn the_wait_doubles_after_each_failure_up_to_five_minutes() {
    let every = Duration::from_secs(30);
    let waits: Vec<u64> = (0..6)
        .map(|failures| next_wait(every, failures).as_secs())
        .collect();
    assert_eq!(waits, vec![30, 60, 120, 240, 300, 300]);
}

#[test]
fn a_period_longer_than_five_minutes_is_never_shortened() {
    let every = Duration::from_secs(600);
    assert_eq!(next_wait(every, 3), every);
}

#[test]
fn the_probe_path_is_joined_to_the_service_url() {
    assert_eq!(
        probe_target("http://10.0.0.5:8096", "/health")
            .unwrap()
            .as_str(),
        "http://10.0.0.5:8096/health"
    );
    assert_eq!(
        probe_target("http://nas.local/app/", "/").unwrap().as_str(),
        "http://nas.local/app/"
    );
    assert_eq!(
        probe_target("http://nas.local/app", "status")
            .unwrap()
            .as_str(),
        "http://nas.local/app/status"
    );
}

mod local_network {
    use std::net::IpAddr;
    use std::time::Duration;

    use ipnet::IpNet;
    use portal_model::{Diagnosis, ServiceState};

    use crate::helpers::{local_network_denied, refine_unreachable};
    use crate::types::{Attempt, Failure};

    fn subnets() -> Vec<IpNet> {
        vec![
            "192.168.1.54/24".parse().unwrap(),
            "10.8.0.102/32".parse().unwrap(),
        ]
    }

    fn attempt(target: &str, milliseconds: u64) -> Attempt {
        Attempt {
            target: Some(target.parse::<IpAddr>().unwrap()),
            elapsed: Duration::from_millis(milliseconds),
        }
    }

    #[test]
    fn an_instant_refusal_inside_a_connected_subnet_is_the_operating_system() {
        assert!(local_network_denied(
            &attempt("192.168.1.60", 1),
            &subnets(),
            true
        ));
    }

    #[test]
    fn an_address_outside_every_connected_subnet_is_a_real_unreachable_host() {
        assert!(!local_network_denied(
            &attempt("172.16.0.9", 1),
            &subnets(),
            true
        ));
    }

    #[test]
    fn a_host_address_route_does_not_count_as_a_connected_subnet() {
        assert!(!local_network_denied(
            &attempt("10.8.0.102", 1),
            &subnets(),
            true
        ));
    }

    #[test]
    fn a_slow_refusal_is_a_host_that_did_not_answer_arp() {
        assert!(!local_network_denied(
            &attempt("192.168.1.60", 2500),
            &subnets(),
            true
        ));
    }

    #[test]
    fn other_platforms_never_blame_the_operating_system() {
        assert!(!local_network_denied(
            &attempt("192.168.1.60", 1),
            &subnets(),
            false
        ));
    }

    #[test]
    fn an_unknown_target_address_is_never_blamed_on_the_operating_system() {
        let unknown = Attempt {
            target: None,
            elapsed: Duration::from_millis(1),
        };
        assert!(!local_network_denied(&unknown, &subnets(), true));
    }

    #[test]
    fn a_denied_unreachable_host_becomes_unreadable_with_the_diagnosis() {
        let outcome = Failure::new(
            ServiceState::Down,
            Diagnosis::HostUnreachable,
            "No route to host (os error 65)",
        )
        .into_outcome(None);
        let refined = refine_unreachable(outcome, true);
        assert_eq!(refined.state, ServiceState::Unreadable);
        assert_eq!(refined.diagnosis, Some(Diagnosis::LocalNetworkDenied));
        assert!(refined.error.unwrap().contains("os error 65"));
    }

    #[test]
    fn a_refused_connection_is_never_refined() {
        let outcome =
            Failure::new(ServiceState::Down, Diagnosis::Refused, "refused").into_outcome(None);
        assert_eq!(refine_unreachable(outcome.clone(), true), outcome);
    }
}

mod echo {
    use std::net::IpAddr;

    use crate::helpers::echo::checksum;
    use crate::helpers::{echo_request, is_echo_reply};

    const TOKEN: [u8; 8] = *b"homeport";

    fn v4() -> IpAddr {
        "127.0.0.1".parse().unwrap()
    }

    #[test]
    fn an_ipv4_echo_request_carries_a_valid_checksum() {
        let packet = echo_request(v4(), 7, TOKEN);
        assert_eq!(packet[0], 8);
        assert_eq!(checksum(&packet), 0);
    }

    #[test]
    fn a_reply_matches_by_sequence_and_token_whatever_the_identifier() {
        let mut reply = echo_request(v4(), 7, TOKEN);
        reply[0] = 0;
        reply[4] = 0xab;
        assert!(is_echo_reply(&reply, 7, TOKEN));
        assert!(!is_echo_reply(&reply, 8, TOKEN));
        assert!(!is_echo_reply(&reply, 7, *b"stranger"));
    }

    #[test]
    fn a_reply_with_the_ip_header_in_front_is_still_recognised() {
        let mut reply = vec![0x45];
        reply.extend(std::iter::repeat_n(0u8, 19));
        let mut message = echo_request(v4(), 3, TOKEN);
        message[0] = 0;
        reply.extend(message);
        assert!(is_echo_reply(&reply, 3, TOKEN));
    }

    #[test]
    fn a_request_echoed_back_unchanged_is_not_a_reply() {
        let request = echo_request(v4(), 3, TOKEN);
        assert!(!is_echo_reply(&request, 3, TOKEN));
    }
}
