#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelReadiness {
    Ready,
    Disabled,
    Missing { field: String, message: String },
}

impl ChannelReadiness {
    pub fn is_ready(&self) -> bool {
        matches!(self, ChannelReadiness::Ready)
    }

    pub fn name(&self) -> &'static str {
        match self {
            ChannelReadiness::Ready => "ready",
            ChannelReadiness::Disabled => "disabled",
            ChannelReadiness::Missing { .. } => "missing",
        }
    }
}
