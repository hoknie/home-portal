use std::sync::Arc;
use std::time::Duration;

use portal_model::Environment;
use time::OffsetDateTime;
use tokio::sync::Notify;

use crate::helpers::next_wait;
use crate::probes::Probe;
use crate::services::StatusBoard;
use crate::types::ServiceEntry;

pub struct Prober {
    pub entry: ServiceEntry,
    pub host: Environment,
    pub probe: Arc<Probe>,
    pub board: Arc<StatusBoard>,
    pub wake: Arc<Notify>,
}

impl Prober {
    pub async fn run(self) {
        let every = Duration::from_secs(self.entry.probe.every_seconds);
        let mut failures = 0u32;
        loop {
            let address = self.entry.probe_address(&self.host).to_string();
            let outcome = self.probe.run(&self.entry, &address).await;
            failures = if outcome.state.failed() {
                failures.saturating_add(1)
            } else {
                0
            };
            self.board
                .record(&self.entry, outcome, OffsetDateTime::now_utc());
            tokio::select! {
                () = tokio::time::sleep(next_wait(every, failures)) => {}
                () = self.wake.notified() => {
                    failures = 0;
                }
            }
        }
    }
}
