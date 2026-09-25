use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceLink {
    pub title: String,
    pub url: String,
}

impl ServiceLink {
    pub const MAXIMUM_TITLE_LENGTH: usize = 80;
}
