use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use portal_feature::{EventName, EventSink, PortalEvent};
use time::OffsetDateTime;

use super::{Gatekeeper, RunQueue, RunningGroups};
use crate::helpers::manual_event;
use crate::services::{AutomationCache, Journal, matching};
use crate::types::{Automation, Pending, RunRecord, SkipReason, Webhook};

pub struct AutomationSink {
    pub cache: Arc<AutomationCache>,
    pub queue: Arc<RunQueue>,
    pub gatekeeper: Arc<Gatekeeper>,
    pub journal: Arc<Journal>,
    pub groups: Arc<RunningGroups>,
    phase: Phase,
    next_run: AtomicU64,
}

#[derive(Default)]
struct Phase {
    stopping: AtomicBool,
    closed: AtomicBool,
}

impl AutomationSink {
    pub const SETTLE_POLL: Duration = Duration::from_millis(50);
    pub const KILL_GRACE: Duration = Duration::from_secs(1);

    #[cfg(test)]
    pub fn new(cache: Arc<AutomationCache>) -> AutomationSink {
        Self::restored(cache, (Vec::new(), 0))
    }

    pub fn restored(
        cache: Arc<AutomationCache>,
        (newest_first, highest): (Vec<RunRecord>, u64),
    ) -> AutomationSink {
        let journal = Journal::default();
        journal.restore(newest_first);
        let next = journal.highest_id().max(highest) + 1;
        AutomationSink {
            cache,
            queue: Arc::new(RunQueue::default()),
            gatekeeper: Arc::new(Gatekeeper::default()),
            journal: Arc::new(journal),
            groups: Arc::new(RunningGroups::default()),
            phase: Phase::default(),
            next_run: AtomicU64::new(next),
        }
    }

    pub fn run_now(&self, automation: &Automation, by: &str) -> u64 {
        let event = manual_event(automation, OffsetDateTime::now_utc());
        self.admit(automation, event, Some(by.to_string()))
    }

    pub fn run_webhook(&self, webhook: &Webhook, event: PortalEvent) -> Option<u64> {
        let automation = webhook.as_automation()?;
        Some(self.admit(&automation, event.aimed_at(webhook.id.clone()), None))
    }

    pub fn stopping(&self) -> bool {
        self.phase.stopping.load(Ordering::SeqCst)
    }

    pub fn closed(&self) -> bool {
        self.phase.closed.load(Ordering::SeqCst)
    }

    fn admit(&self, automation: &Automation, event: PortalEvent, by: Option<String>) -> u64 {
        let run_id = self.next_run.fetch_add(1, Ordering::SeqCst);
        let pending = Pending {
            run_id,
            automation: automation.clone(),
            event,
            by,
        };
        let now = OffsetDateTime::now_utc();
        if self.stopping() && pending.event.name != EventName::PortalStopping {
            self.journal
                .record(RunRecord::skipped(&pending, SkipReason::Dropped, now));
            return run_id;
        }
        match self.gatekeeper.admit(automation, Instant::now()) {
            Err(reason) => self
                .journal
                .record(RunRecord::skipped(&pending, reason, now)),
            Ok(()) => {
                if let Some(dropped) = self.queue.push(pending) {
                    self.gatekeeper.dequeued(&dropped.automation.id);
                    self.journal
                        .record(RunRecord::skipped(&dropped, SkipReason::Dropped, now));
                }
            }
        }
        run_id
    }

    fn busy(&self) -> bool {
        !self.queue.is_empty() || self.gatekeeper.busy() || self.groups.count() > 0
    }
}

#[async_trait]
impl EventSink for AutomationSink {
    fn emit(&self, event: PortalEvent) {
        if self.stopping() {
            return;
        }
        let stopping = event.name == EventName::PortalStopping;
        let automations = self.cache.automations();
        for automation in matching(&automations, &event) {
            self.admit(automation, event.clone(), None);
        }
        if stopping {
            self.phase.stopping.store(true, Ordering::SeqCst);
        }
    }

    async fn settle(&self, within: Duration) {
        self.phase.stopping.store(true, Ordering::SeqCst);
        let deadline = Instant::now() + within;
        while self.busy() && Instant::now() < deadline {
            tokio::time::sleep(Self::SETTLE_POLL).await;
        }
        self.phase.closed.store(true, Ordering::SeqCst);
        let now = OffsetDateTime::now_utc();
        while let Some(dropped) = self.queue.take() {
            self.gatekeeper.dequeued(&dropped.automation.id);
            self.journal
                .record(RunRecord::skipped(&dropped, SkipReason::Dropped, now));
        }
        let grace = Instant::now() + Self::KILL_GRACE;
        loop {
            self.groups.kill_all();
            if (self.groups.count() == 0 && !self.gatekeeper.busy()) || Instant::now() >= grace {
                return;
            }
            tokio::time::sleep(Self::SETTLE_POLL).await;
        }
    }
}
