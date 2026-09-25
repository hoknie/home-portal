#[derive(Debug, Clone)]
pub struct Fetched {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}
