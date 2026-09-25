use std::sync::Arc;
use std::time::Duration;

use time::OffsetDateTime;

use crate::services::Icons;

pub const EVERY: Duration = Duration::from_secs(60 * 60);
pub const SETTLE: Duration = Duration::from_secs(2);

pub async fn refresh_forever(icons: Arc<Icons>) {
    tokio::time::sleep(SETTLE).await;
    loop {
        icons.refresh_all(OffsetDateTime::now_utc()).await;
        tokio::time::sleep(EVERY).await;
    }
}
