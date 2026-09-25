#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusChange {
    pub service: String,
    pub name: String,
    pub was: String,
    pub now: String,
    pub error: Option<String>,
}

impl StatusChange {
    pub const UNKNOWN: &'static str = "unknown";

    pub fn from_unknown(&self) -> bool {
        self.was == Self::UNKNOWN
    }
}
