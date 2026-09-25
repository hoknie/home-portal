#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredIcon {
    pub bytes: Vec<u8>,
    pub content_type: String,
    pub digest: String,
}
