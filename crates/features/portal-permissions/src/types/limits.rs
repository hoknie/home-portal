use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub network: Duration,
    pub prompt: Duration,
    pub diskutil: Duration,
    pub retry: Duration,
}

impl Limits {
    pub const NETWORK: Duration = Duration::from_secs(10);
    pub const PROMPT: Duration = Duration::from_secs(60);
    pub const DISKUTIL: Duration = Duration::from_secs(5);
    pub const RETRY: Duration = Duration::from_secs(2);
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            network: Self::NETWORK,
            prompt: Self::PROMPT,
            diskutil: Self::DISKUTIL,
            retry: Self::RETRY,
        }
    }
}
