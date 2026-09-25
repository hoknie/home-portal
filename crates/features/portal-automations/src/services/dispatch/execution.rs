use std::sync::Arc;
use std::time::Duration;

use time::OffsetDateTime;

use super::AutomationSink;
use crate::clients::Runner;
use crate::services::ScriptsDirectory;
use crate::types::{Finished, Invocation, Outcome, Pending, RunRecord, Tail};

pub async fn execute(sink: Arc<AutomationSink>, scripts: &ScriptsDirectory, pending: Pending) {
    if sink.closed() {
        return;
    }
    let started_at = OffsetDateTime::now_utc();
    let record = match scripts.resolve(&pending.automation.run.script) {
        Err(refusal) => RunRecord::finished(
            &pending,
            Vec::new(),
            started_at,
            Finished {
                outcome: Outcome::Refused,
                exit_code: None,
                reason: Some(refusal.message),
                duration: Duration::ZERO,
                stdout: Tail::default(),
                stderr: Tail::default(),
            },
        ),
        Ok(program) => {
            let directory = scripts.canonical_root();
            let invocation = Invocation::for_run(&pending, program, directory);
            let finished = Runner::run(&invocation, sink.groups.clone()).await;
            RunRecord::finished(&pending, invocation.arguments, started_at, finished)
        }
    };
    sink.journal.record(record);
}
