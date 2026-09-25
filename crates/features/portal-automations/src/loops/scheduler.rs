use std::sync::Arc;
use std::time::Duration;

use portal_feature::{EventName, EventSink, PortalEvent};

use crate::helpers::offset_of;
use crate::ports::Clock;
use crate::services::{AutomationSink, ScheduleBook};

pub const LONGEST_SLEEP: Duration = Duration::from_secs(2);

pub async fn schedule_forever(sink: Arc<AutomationSink>, clock: Arc<dyn Clock>) {
    let mut book = ScheduleBook::default();
    loop {
        let automations = sink.cache.automations();
        let zone = sink.cache.zone();
        let now = clock.now();
        for (id, expression, due) in book.due(&automations, &zone, now) {
            let event = PortalEvent::of(
                EventName::Schedule,
                offset_of(now),
                &[
                    ("schedule.cron", expression.as_str()),
                    ("schedule.at", &PortalEvent::timestamp(offset_of(due))),
                ],
            )
            .aimed_at(id);
            sink.emit(event);
        }
        let wait = book
            .earliest()
            .and_then(|next| Duration::try_from(next.duration_since(clock.now())).ok())
            .map_or(LONGEST_SLEEP, |wait| wait.min(LONGEST_SLEEP));
        tokio::select! {
            _ = tokio::time::sleep(wait.max(Duration::from_millis(10))) => {}
            _ = sink.cache.changed() => {}
        }
    }
}
