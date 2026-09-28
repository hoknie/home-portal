use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::json;
use tokio::sync::watch;

use super::support::section;
pub use crate::fakes::{FakeActions, FakeStarter};
use crate::services::workflow::{
    Budget, Frame, Secrets, WorkflowRunner, decoded_workflows, workflow_errors,
};
use crate::types::{Ending, StepOutcome, Trace};

pub struct Outcome {
    pub ending: Ending,
    pub trace: Trace,
    pub actions: Arc<FakeActions>,
    pub starter: Arc<FakeStarter>,
    pub frame: Frame,
}

#[derive(Default)]
pub struct Groups {
    pub alive: Mutex<Vec<u32>>,
}

impl crate::clients::GroupRegistry for Groups {
    fn started(&self, group: u32) {
        self.alive.lock().unwrap().push(group);
    }

    fn reaped(&self, group: u32) {
        self.alive.lock().unwrap().retain(|alive| *alive != group);
    }
}

pub async fn run_with(
    text: &str,
    actions: Arc<FakeActions>,
    stop: watch::Receiver<bool>,
) -> Outcome {
    run_scripted(text, (actions, stop), &[]).await
}

pub async fn run_scripted(
    text: &str,
    (actions, stop): (Arc<FakeActions>, watch::Receiver<bool>),
    scripts: &[(&str, &str)],
) -> Outcome {
    run_prepared(text, (actions, stop), scripts, Secrets::none()).await
}

pub async fn run_prepared(
    text: &str,
    (actions, stop): (Arc<FakeActions>, watch::Receiver<bool>),
    scripts: &[(&str, &str)],
    secrets: Secrets,
) -> Outcome {
    let folder = tempfile::tempdir().unwrap();
    let root = folder.path().join("scripts");
    std::fs::create_dir(&root).unwrap();
    std::fs::set_permissions(
        &root,
        <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o755),
    )
    .unwrap();
    for (name, body) in scripts {
        crate::features::tests::write_script(&root, name, body);
    }
    let section = section(text);
    assert!(
        workflow_errors(&section).is_empty(),
        "{:?}",
        workflow_errors(&section)
    );
    let workflows = Arc::new(decoded_workflows(&section));
    let workflow = workflows[0].clone();
    let starter = Arc::new(FakeStarter::default());
    let runner = WorkflowRunner {
        workflows,
        actions: actions.clone(),
        http: crate::clients::HttpClient::new().unwrap(),
        scripts: crate::services::ScriptsDirectory::at(root),
        groups: Arc::new(Groups::default()),
        budget: Budget::new(Duration::from_secs(workflow.timeout_seconds), stop),
        trace: Arc::new(Mutex::new(Trace::default())),
        starter: starter.clone(),
        chain: vec!["nas-down".to_string()],
        switches: portal_feature::ModuleSwitches::default(),
        logging: crate::types::StepLogging::On,
    };
    let mut frame = Frame::new(
        Arc::new(vec![("automation.id".to_string(), "w".to_string())]),
        crate::services::workflow::bind_inputs(
            &workflow,
            vec![("service".to_string(), serde_json::json!("nas"))],
        )
        .unwrap(),
        Arc::new(secrets),
    );
    let ending = runner.run(&workflow, &mut frame).await;
    let trace = runner.trace().clone();
    drop(folder);
    Outcome {
        ending,
        trace,
        actions,
        starter,
        frame,
    }
}

pub async fn run(text: &str) -> Outcome {
    let (_sender, stop) = watch::channel(false);
    run_with(text, Arc::new(FakeActions::default()), stop).await
}

fn workflow(steps: &str) -> String {
    format!("[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n{steps}")
}

#[tokio::test]
async fn a_branch_is_chosen_and_the_other_does_not_run() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"check\"\nkind = \"status\"\nservice = \"{{inputs.service}}\"\n[[workflows.steps]]\nid = \"down\"\nkind = \"if\"\ncondition = { left = \"{{steps.check.state}}\", op = \"==\", right = \"down\" }\nthen = [{ id = \"yes\", kind = \"telegram\", text = \"{{inputs.service}} is down\" }]\nelse = [{ id = \"no\", kind = \"telegram\", text = \"fine\" }]\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(*outcome.actions.sent.lock().unwrap(), vec!["nas is down"]);
    assert_eq!(outcome.frame.steps["down"], json!({"branch": "then"}));
    let paths: Vec<&str> = outcome
        .trace
        .entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    assert_eq!(paths, vec!["steps[0]", "steps[1]", "steps[1].then[0]"]);
    assert_eq!(outcome.trace.entries[1].detail, "then");
}

#[tokio::test]
async fn a_loop_that_never_ends_fails_at_its_bound() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"forever\"\nkind = \"loop\"\nwhile = { left = \"1\", op = \"==\", right = \"1\" }\nmax_iterations = 50\nbody = [{ id = \"count\", kind = \"set\", variable = \"n\", value = \"{{loop.index}}\" }]\n")).await;
    let Ending::Failed(reason) = &outcome.ending else {
        panic!("{:?}", outcome.ending);
    };
    assert!(reason.contains("max_iterations (50)"), "{reason}");
    assert_eq!(outcome.frame.vars["n"], json!(49));
    let entry = &outcome.trace.entries[0];
    assert_eq!(entry.step, "forever");
    assert_eq!(entry.outcome, StepOutcome::Failed);
    assert!(entry.detail.contains("max_iterations"), "{}", entry.detail);
}

#[tokio::test]
async fn stopping_a_run_ends_a_long_wait_at_once() {
    let (sender, stop) = watch::channel(false);
    let text = workflow(
        "[[workflows.steps]]\nid = \"nap\"\nkind = \"wait\"\nseconds = 600\n[[workflows.steps]]\nid = \"after\"\nkind = \"telegram\"\ntext = \"never\"\n",
    );
    let began = Instant::now();
    let running =
        tokio::spawn(async move { run_with(&text, Arc::new(FakeActions::default()), stop).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    sender.send(true).unwrap();
    let outcome = running.await.unwrap();
    assert_eq!(outcome.ending, Ending::Stopped);
    assert!(began.elapsed() < Duration::from_secs(2));
    assert!(outcome.actions.sent.lock().unwrap().is_empty());
    assert_eq!(outcome.trace.entries[0].outcome, StepOutcome::Stopped);
}

#[tokio::test]
async fn more_than_a_thousand_steps_fail_the_run() {
    let body: Vec<String> = (0..11)
        .map(|index| {
            format!("{{ id = \"s{index}\", kind = \"set\", variable = \"v\", value = \"x\" }}")
        })
        .collect();
    let outcome = run(&workflow(&format!(
        "[[workflows.steps]]\nid = \"many\"\nkind = \"loop\"\nrepeat = 100\nbody = [{}]\n",
        body.join(", ")
    )))
    .await;
    assert_eq!(
        outcome.ending,
        Ending::Failed("ran more than 1000 steps".into())
    );
    assert_eq!(outcome.trace.entries.len(), Trace::MOST_ENTRIES);
    assert!(outcome.trace.dropped > 700);
}

#[tokio::test]
async fn parallel_branches_merge_in_branch_order() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"both\"\nkind = \"parallel\"\nbranches = [\n  [{ id = \"a\", kind = \"set\", variable = \"who\", value = \"first\" }, { id = \"slow\", kind = \"wait\", seconds = 1 }],\n  [{ id = \"b\", kind = \"set\", variable = \"who\", value = \"second\" }],\n]\n[[workflows.steps]]\nid = \"say\"\nkind = \"telegram\"\ntext = \"{{vars.who}} {{steps.a.value}} {{steps.b.value}}\"\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(
        *outcome.actions.sent.lock().unwrap(),
        vec!["second first second"]
    );
}

#[tokio::test]
async fn a_trace_of_a_retry_shows_every_iteration() {
    let actions = Arc::new(FakeActions::default());
    actions.states.lock().unwrap().extend([
        "up".to_string(),
        "down".to_string(),
        "down".to_string(),
    ]);
    let (_sender, stop) = watch::channel(false);
    let outcome = run_with(&workflow("[[workflows.steps]]\nid = \"retry\"\nkind = \"loop\"\nrepeat = 5\n[[workflows.steps.body]]\nid = \"try\"\nkind = \"probe\"\nservice = \"nas\"\n[[workflows.steps.body]]\nid = \"back\"\nkind = \"if\"\ncondition = { left = \"{{steps.try.state}}\", op = \"==\", right = \"up\" }\nthen = [{ id = \"done\", kind = \"stop\", outcome = \"succeeded\", reason = \"back after {{loop.index}}\" }]\n"), actions, stop).await;
    assert_eq!(
        outcome.ending,
        Ending::Succeeded(Some("back after 2".into()))
    );
    let tries: Vec<(Option<usize>, &str)> = outcome
        .trace
        .entries
        .iter()
        .filter(|entry| entry.step == "try")
        .map(|entry| (entry.iteration, entry.detail.as_str()))
        .collect();
    assert_eq!(
        tries,
        vec![
            (Some(0), "nas: down"),
            (Some(1), "nas: down"),
            (Some(2), "nas: up")
        ]
    );
}

#[tokio::test]
async fn a_called_workflow_returns_its_variables_and_a_failing_action_fails_the_run() {
    let text = "[[workflows]]\nid = \"caller\"\ntitle = \"C\"\n[[workflows.steps]]\nid = \"go\"\nkind = \"workflow\"\nworkflow = \"helper\"\ninputs = { name = \"nas\" }\n[[workflows.steps]]\nid = \"say\"\nkind = \"telegram\"\ntext = \"{{steps.go.vars.greeting}}\"\n[[workflows.steps]]\nid = \"broken\"\nkind = \"probe\"\nservice = \"ghost\"\n\n[[workflows]]\nid = \"helper\"\ntitle = \"H\"\ninputs = [\"name\"]\n[[workflows.steps]]\nid = \"greet\"\nkind = \"set\"\nvariable = \"greeting\"\nvalue = \"hello {{inputs.name}}\"\n";
    let outcome = run(text).await;
    assert_eq!(*outcome.actions.sent.lock().unwrap(), vec!["hello nas"]);
    assert_eq!(outcome.ending, Ending::Failed("no service ghost".into()));
    assert!(
        outcome
            .trace
            .entries
            .iter()
            .any(|entry| entry.path == "steps[0].helper.steps[0]")
    );
}

#[tokio::test]
async fn a_branch_that_does_nothing_runs_and_the_run_goes_on() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"check\"\nkind = \"if\"\ncondition = { left = \"a\", op = \"==\", right = \"a\" }\nthen = [{ id = \"skip\", kind = \"nothing\" }]\nelse = [{ id = \"tell\", kind = \"telegram\", text = \"x\" }]\n[[workflows.steps]]\nid = \"after\"\nkind = \"telegram\"\ntext = \"after\"\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(*outcome.actions.sent.lock().unwrap(), vec!["after"]);
    assert_eq!(outcome.trace.entries[1].step, "skip");
    assert_eq!(outcome.trace.entries[1].outcome, StepOutcome::Succeeded);
}

#[tokio::test]
async fn a_notify_step_names_its_deliveries_and_an_unknown_channel_fails_it() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"tell\"\nkind = \"notify\"\ntitle = \"NAS\"\ntext = \"{{inputs.service}} is back\"\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(*outcome.actions.sent.lock().unwrap(), vec!["nas is back"]);
    assert_eq!(outcome.trace.entries[0].detail, "delivered: telegram");
    let failed = run(&workflow(
        "[[workflows.steps]]\nid = \"tell\"\nkind = \"notify\"\nchannel = \"sms\"\ntext = \"x\"\n",
    ))
    .await;
    assert!(
        matches!(failed.ending, Ending::Failed(ref reason) if reason.contains("sms")),
        "{:?}",
        failed.ending
    );
}
