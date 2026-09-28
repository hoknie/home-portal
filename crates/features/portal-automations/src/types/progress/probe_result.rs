#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeResult {
    pub state: String,
    pub latency_milliseconds: Option<u64>,
    pub diagnosis: Option<String>,
}
