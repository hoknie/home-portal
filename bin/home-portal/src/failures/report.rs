use std::net::SocketAddr;

use axum::http::HeaderMap;

use super::board::FailureBoard;
use super::problems::Problem;
use super::visibility::sees_details;
use crate::responses::{FailureReportResponse, ProblemResponse};

pub fn report(
    board: &FailureBoard,
    peer: Option<SocketAddr>,
    headers: &HeaderMap,
) -> FailureReportResponse {
    let failure = board.current();
    let details = sees_details(peer, headers, &failure.main);
    FailureReportResponse {
        since: failure.since,
        checked: failure.checked,
        details,
        problems: details.then(|| failure.problems.iter().map(problem_response).collect()),
    }
}

fn problem_response(problem: &Problem) -> ProblemResponse {
    ProblemResponse {
        file: problem.file.clone(),
        field: problem.field.clone(),
        message: problem.message.clone(),
    }
}
