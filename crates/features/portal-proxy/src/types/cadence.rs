use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cadence {
    pub tick: Duration,
    pub check_every: Duration,
    pub retry_every: Duration,
    pub restart_every: Duration,
}

impl Cadence {
    pub const STANDARD: Cadence = Cadence {
        tick: Duration::from_secs(1),
        check_every: Duration::from_secs(60),
        retry_every: Duration::from_secs(5),
        restart_every: Duration::from_secs(30),
    };
}
