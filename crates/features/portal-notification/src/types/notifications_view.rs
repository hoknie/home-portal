use super::{ChannelView, Rules};

#[derive(Debug, Clone)]
pub struct NotificationsView {
    pub enabled: bool,
    pub rules: Rules,
    pub channels: Vec<ChannelView>,
}
