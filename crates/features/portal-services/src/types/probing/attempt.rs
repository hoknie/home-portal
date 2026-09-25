use std::net::IpAddr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attempt {
    pub target: Option<IpAddr>,
    pub elapsed: Duration,
}
