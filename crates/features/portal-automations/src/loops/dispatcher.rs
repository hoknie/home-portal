use std::sync::Arc;
use std::time::Instant;

use time::OffsetDateTime;
use tokio::sync::Semaphore;

use crate::ports::ScriptLibrary;
use crate::services::{AutomationSink, FinishGuard, WorkflowTools, execute};
use crate::types::{RunRecord, SkipReason};

pub const PARALLEL_RUNS: usize = 4;

pub async fn dispatch_forever(
    sink: Arc<AutomationSink>,
    scripts: Arc<dyn ScriptLibrary>,
    tools: WorkflowTools,
) {
    let permits = Arc::new(Semaphore::new(PARALLEL_RUNS));
    loop {
        let Ok(permit) = permits.clone().acquire_owned().await else {
            return;
        };
        let pending = sink.queue.next().await;
        let id = pending.automation.id.clone();
        if sink.closed() {
            sink.forget(&pending);
            continue;
        }
        if !sink.cache.runnable(&id) {
            sink.forget(&pending);
            sink.journal.record(RunRecord::skipped(
                &pending,
                SkipReason::Removed,
                OffsetDateTime::now_utc(),
            ));
            continue;
        }
        sink.gatekeeper.started(&id, Instant::now());
        let guard = FinishGuard {
            gatekeeper: sink.gatekeeper.clone(),
            automation: id,
        };
        let sink = sink.clone();
        let scripts = scripts.clone();
        let tools = tools.clone();
        tokio::spawn(async move {
            let _guard = guard;
            let _permit = permit;
            execute(sink, (&scripts, &tools), pending).await;
        });
    }
}

pub async fn dispatch_children_forever(
    sink: Arc<AutomationSink>,
    scripts: Arc<dyn ScriptLibrary>,
    tools: WorkflowTools,
) {
    loop {
        let pending = sink.children.next().await;
        let id = pending.automation.id.clone();
        if sink.closed() || !sink.cache.runnable(&id) {
            sink.forget(&pending);
            sink.journal.record(RunRecord::skipped(
                &pending,
                SkipReason::Removed,
                OffsetDateTime::now_utc(),
            ));
            continue;
        }
        sink.gatekeeper.started(&id, Instant::now());
        let guard = FinishGuard {
            gatekeeper: sink.gatekeeper.clone(),
            automation: id,
        };
        let sink = sink.clone();
        let scripts = scripts.clone();
        let tools = tools.clone();
        tokio::spawn(async move {
            let _guard = guard;
            execute(sink, (&scripts, &tools), pending).await;
        });
    }
}
