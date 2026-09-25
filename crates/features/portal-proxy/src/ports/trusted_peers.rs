use std::net::IpAddr;

pub trait TrustedPeers: Send + Sync {
    fn trusts(&self, peer: IpAddr) -> bool;
}
