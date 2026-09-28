use serde_json::Value;

use super::join::{Pending, join_all};
use super::runner::WorkflowRunner;
use crate::services::workflow::evaluating::{
    Frame, bind_inputs, holds, input_problem, render_value,
};
use crate::types::{Ending, Flow, LoopMode, Place, Step, StepKind, StepReport, Workflow};

pub async fn run_if(
    runner: &WorkflowRunner,
    step: &Step,
    frame: &mut Frame,
    place: &Place,
) -> StepReport {
    let StepKind::If {
        condition,
        then,
        otherwise,
    } = &step.kind
    else {
        return StepReport::failed("not an if step");
    };
    let taken = match holds(condition, frame) {
        Ok(true) => "then",
        Ok(false) => "else",
        Err(message) => return StepReport::failed(message),
    };
    let branch = if taken == "then" { then } else { otherwise };
    let flow = runner.run_steps(branch, frame, &place.inside(taken)).await;
    StepReport::done(
        WorkflowRunner::step_result(&[("branch", Value::from(taken))]),
        taken,
    )
    .with_flow(flow)
}

pub async fn run_loop(
    runner: &WorkflowRunner,
    step: &Step,
    frame: &mut Frame,
    place: &Place,
) -> StepReport {
    let StepKind::Loop {
        mode,
        max_iterations,
        body,
    } = &step.kind
    else {
        return StepReport::failed("not a loop step");
    };
    let items = match mode {
        LoopMode::ForEach(list) => match render_value(list, frame) {
            Ok(Value::Array(items)) => Some(items),
            Ok(_) => return StepReport::failed("for_each does not render a JSON list"),
            Err(message) => return StepReport::failed(message),
        },
        _ => None,
    };
    let previous = frame.loop_item.take();
    let inside = place.inside("body");
    let mut count = 0usize;
    let mut flow = Flow::Continue;
    loop {
        let more = match mode {
            LoopMode::Repeat(times) => count < *times as usize,
            LoopMode::ForEach(_) => count < items.as_ref().map_or(0, Vec::len),
            LoopMode::While(condition) => match holds(condition, frame) {
                Ok(more) => more,
                Err(message) => {
                    flow = Flow::End(Ending::Failed(message));
                    break;
                }
            },
        };
        if !more {
            break;
        }
        if count >= *max_iterations as usize {
            flow = Flow::End(Ending::Failed(format!(
                "ran more than max_iterations ({max_iterations}) iterations"
            )));
            break;
        }
        let item = items
            .as_ref()
            .and_then(|items| items.get(count).cloned())
            .unwrap_or_else(|| Value::from(count));
        frame.loop_item = Some((item, count));
        let result = runner.run_steps(body, frame, &inside.repeated(count)).await;
        count += 1;
        if let Flow::End(ending) = result {
            flow = Flow::End(ending);
            break;
        }
    }
    frame.loop_item = previous;
    let detail = match &flow {
        Flow::End(Ending::Failed(reason)) => reason.clone(),
        _ => format!("{count} iterations"),
    };
    StepReport::done(
        WorkflowRunner::step_result(&[("iterations", Value::from(count))]),
        detail,
    )
    .with_flow(flow)
}

pub async fn run_parallel(
    runner: &WorkflowRunner,
    branches: &[Vec<Step>],
    frame: &mut Frame,
    place: &Place,
) -> StepReport {
    let mut frames: Vec<Frame> = branches.iter().map(|_| frame.clone()).collect();
    let places: Vec<Place> = (0..branches.len())
        .map(|index| place.inside(&format!("branches[{index}]")))
        .collect();
    let futures: Vec<Pending<'_, Flow>> = branches
        .iter()
        .zip(frames.iter_mut())
        .zip(places.iter())
        .map(|((branch, branch_frame), branch_place)| {
            runner.run_steps(branch, branch_frame, branch_place)
        })
        .collect();
    let flows = join_all(futures).await;
    for branch_frame in frames {
        frame.vars.extend(branch_frame.vars);
        frame.steps.extend(branch_frame.steps);
    }
    let flow = flows
        .into_iter()
        .find(|flow| matches!(flow, Flow::End(_)))
        .unwrap_or(Flow::Continue);
    StepReport::done(Value::Null, format!("{} branches", branches.len())).with_flow(flow)
}

pub async fn run_call(
    runner: &WorkflowRunner,
    step: &Step,
    frame: &mut Frame,
    place: &Place,
) -> StepReport {
    let StepKind::Call { workflow, inputs } = &step.kind else {
        return StepReport::failed("not a workflow step");
    };
    let Some(called) = runner
        .workflows
        .iter()
        .find(|candidate| candidate.id == *workflow)
    else {
        return StepReport::failed(format!("no workflow {workflow}"));
    };
    if !called.enabled {
        return StepReport::failed(format!("the workflow {workflow} is disabled"));
    }
    if place.calls >= Workflow::DEEPEST_CALLS {
        return StepReport::failed(format!(
            "calls nest more than {} deep",
            Workflow::DEEPEST_CALLS
        ));
    }
    let mut given = Vec::new();
    for (name, template) in inputs {
        match render_value(template, frame) {
            Ok(value) => given.push((name.clone(), value)),
            Err(message) => return StepReport::failed(message),
        }
    }
    let given = match bind_inputs(called, given) {
        Ok(given) => given,
        Err(problem) => return StepReport::failed(input_problem(problem)),
    };
    let mut child = Frame::new(frame.event.clone(), given, frame.secrets.clone());
    let flow = runner
        .run_steps(&called.steps, &mut child, &place.called(workflow))
        .await;
    let vars = Value::Object(child.vars.into_iter().collect());
    match flow {
        Flow::Continue | Flow::End(Ending::Succeeded(_)) => StepReport::done(
            WorkflowRunner::step_result(&[("vars", vars)]),
            workflow.clone(),
        ),
        Flow::End(ending) => StepReport::ended(ending),
    }
}
