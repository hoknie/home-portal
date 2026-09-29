use std::time::Duration;

use serde_json::Value;

use super::runner::WorkflowRunner;
use crate::helpers::shape_of;
use crate::services::workflow::evaluating::{Frame, render_keys, render_number, render_text};
use crate::types::{HttpAnswer, HttpRequest, HttpStep, StepReport, TraceEntry};

pub async fn run_http(runner: &WorkflowRunner, step: &HttpStep, frame: &Frame) -> StepReport {
    let request = match rendered(step, frame, runner.budget.remaining()) {
        Ok(request) => request,
        Err(message) => return StepReport::failed(message),
    };
    runner.publish(frame, None);
    let began = std::time::Instant::now();
    let answer = match runner.budget.guard(runner.http.send(&request)).await {
        Ok(Ok(answer)) => answer,
        Ok(Err(message)) => {
            return StepReport::failed(format!("{} {}: {message}", request.method, request.url))
                .logged(format!("{} {} → {message}", request.method, request.url));
        }
        Err(ending) => return StepReport::ended(ending),
    };
    let detail = format!("{} {} → {}", request.method, request.url, answer.status);
    let line = format!("{detail} in {} ms", began.elapsed().as_millis());
    let output = Some(
        answer
            .body
            .chars()
            .take(TraceEntry::LONGEST_OUTPUT)
            .collect(),
    );
    let shape = answer.json.as_ref().map(shape_of);
    let result = result_of(&answer);
    if step.fail_on_error && answer.status >= 400 {
        let mut report = StepReport::failed(detail).logged(line);
        report.result = result;
        report.output = output;
        report.shape = shape;
        return report;
    }
    StepReport {
        output,
        shape,
        ..StepReport::done(result, detail).logged(line)
    }
}

fn rendered(step: &HttpStep, frame: &Frame, remaining: Duration) -> Result<HttpRequest, String> {
    let mut headers = Vec::new();
    for (name, value) in render_keys(&step.headers, frame)? {
        if reqwest::header::HeaderName::from_bytes(name.as_bytes()).is_err() {
            return Err(format!("the header name \"{name}\" is not valid"));
        }
        headers.push((name, render_text(value, frame)?));
    }
    Ok(HttpRequest {
        method: step.method.clone(),
        url: render_text(&step.url, frame)?,
        headers,
        body: step
            .body
            .as_deref()
            .map(|body| render_text(body, frame))
            .transpose()?,
        timeout: Duration::from_secs(render_number(
            &step.timeout_seconds,
            (1, HttpStep::LONGEST_TIMEOUT),
            "timeout_seconds",
            frame,
        )?)
        .min(remaining.max(Duration::from_millis(1))),
    })
}

fn result_of(answer: &HttpAnswer) -> Value {
    let headers: serde_json::Map<String, Value> = answer
        .headers
        .iter()
        .map(|(name, value)| (name.to_ascii_lowercase(), Value::from(value.clone())))
        .collect();
    WorkflowRunner::step_result(&[
        ("status", Value::from(answer.status)),
        ("headers", Value::Object(headers)),
        ("body", Value::from(answer.body.clone())),
        ("json", answer.json.clone().unwrap_or(Value::Null)),
    ])
}
