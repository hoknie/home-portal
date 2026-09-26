use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cadence {
    pub tick: Duration,
    pub retry: Duration,
    pub interfaces: Duration,
    pub idle: Duration,
}

impl Cadence {
    pub const STANDARD: Cadence = Cadence {
        tick: Duration::from_secs(2),
        retry: Duration::from_secs(30),
        interfaces: Duration::from_secs(30),
        idle: Duration::from_secs(10),
    };
    pub const CONNECTIONS: usize = 64;
    pub const LARGEST_MESSAGE: usize = 4096;
}
