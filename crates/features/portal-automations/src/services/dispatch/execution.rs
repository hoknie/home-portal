use std::sync::Arc;
use std::time::Duration;

use time::OffsetDateTime;

use super::AutomationSink;
use super::execute_workflow;
use crate::clients::Runner;
use crate::ports::ScriptLibrary;
use crate::services::WorkflowTools;
use crate::types::{Finished, Invocation, Outcome, Pending, RunControl, RunRecord, Tail};

pub async fn execute(
    sink: Arc<AutomationSink>,
    (scripts, tools): (&Arc<dyn ScriptLibrary>, &WorkflowTools),
    pending: Pending,
) {
    if sink.closed() {
        sink.active.remove(pending.run_id);
        return;
    }
    if let Some(call) = pending.automation.workflow.clone() {
        execute_workflow(sink, (scripts, tools), pending, &call).await;
        return;
    }
    let started_at = OffsetDateTime::now_utc();
    let control = sink
        .active
        .control(pending.run_id)
        .unwrap_or_else(|| RunControl::new().1);
    let mut record = match scripts.resolve(&pending.automation.run.script) {
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
            let directory = scripts.root();
            let invocation = Invocation::for_run(&pending, program, directory);
            if let Some(reason) = Invocation::smuggled_option(&pending, &invocation.arguments) {
                let record = RunRecord::finished(
                    &pending,
                    invocation.arguments,
                    started_at,
                    Finished {
                        outcome: Outcome::Refused,
                        exit_code: None,
                        reason: Some(reason),
                        duration: Duration::ZERO,
                        stdout: Tail::default(),
                        stderr: Tail::default(),
                    },
                );
                sink.journal.record(record);
                sink.active.remove(pending.run_id);
                return;
            }
            sink.active
                .started(pending.run_id, invocation.arguments.clone(), started_at);
            let finished = Runner::run(&invocation, sink.groups.clone(), control).await;
            RunRecord::finished(&pending, invocation.arguments, started_at, finished)
        }
    };
    if record.result.outcome == Outcome::Stopped {
        let by = sink.active.stopped_by(pending.run_id).unwrap_or_default();
        record.result.reason = Some(RunRecord::stopped_reason(&by));
    }
    sink.journal.record(record);
    sink.active.remove(pending.run_id);
}
