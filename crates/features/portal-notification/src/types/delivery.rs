use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivery {
    pub channel: String,
    pub at: OffsetDateTime,
    pub error: Option<String>,
}

impl Delivery {
    pub fn succeeded(&self) -> bool {
        self.error.is_none()
    }
}
