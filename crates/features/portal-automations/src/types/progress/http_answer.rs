use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct HttpAnswer {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub json: Option<Value>,
}

impl HttpAnswer {
    pub const LARGEST_READ: usize = 10 * 1024 * 1024;
    pub const LARGEST_BODY: usize = 64 * 1024;
    pub const MOST_REDIRECTS: usize = 5;
}
