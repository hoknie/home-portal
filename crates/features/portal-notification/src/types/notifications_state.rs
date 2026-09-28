use crate::usecases::{ChangeChannel, ChangeRules, ListNotifications, SendTest};

#[derive(Clone)]
pub struct NotificationsState {
    pub list: ListNotifications,
    pub change_rules: ChangeRules,
    pub change_channel: ChangeChannel,
    pub send_test: SendTest,
}
