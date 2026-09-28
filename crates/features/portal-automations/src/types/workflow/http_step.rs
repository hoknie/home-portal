#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpStep {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub timeout_seconds: u64,
    pub fail_on_error: bool,
}

impl HttpStep {
    pub const DEFAULT_TIMEOUT: u64 = 10;
    pub const LONGEST_TIMEOUT: u64 = 60;
    pub const FORBIDDEN_HEADERS: [&'static str; 2] = ["host", "content-length"];
    pub const LARGEST_SAMPLE: usize = 16 * 1024;
}
