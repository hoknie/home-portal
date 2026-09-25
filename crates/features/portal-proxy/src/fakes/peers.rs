use std::net::IpAddr;

use crate::ports::TrustedPeers;

pub struct Loopback;

impl TrustedPeers for Loopback {
    fn trusts(&self, peer: IpAddr) -> bool {
        peer.is_loopback()
    }
}
