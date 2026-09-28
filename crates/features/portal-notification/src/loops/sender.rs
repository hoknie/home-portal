use std::sync::Arc;

use crate::services::ChannelSet;

pub async fn send_forever(channels: ChannelSet, index: usize) {
    let Some(entry) = channels.channels.get(index) else {
        return;
    };
    let outbox = Arc::clone(&entry.outbox);
    loop {
        let notification = outbox.next().await;
        channels.deliver(entry, &notification).await;
    }
}
