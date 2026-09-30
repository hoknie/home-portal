use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinError;

use super::board::PermissionBoard;
use super::report::{logged, view_of};
use crate::ports::PermissionCheck;
use crate::types::{Advice, Finding, Owner, PermissionCode};

pub async fn asked(
    check: Arc<dyn PermissionCheck>,
    board: PermissionBoard,
    limit: Duration,
    owner: Owner,
) {
    let code = check.code();
    if !board.begin(&code) {
        return;
    }
    if code.requestable() {
        board.record(code.clone(), Finding::pending());
    }
    let mut task = tokio::task::spawn_blocking(move || check.ask());
    match tokio::time::timeout(limit, &mut task).await {
        Ok(result) => settled(&board, code, result, &owner),
        Err(_) => {
            logged(&view_of(code.clone(), board.get(&code)), &owner);
            tokio::spawn(async move {
                let result = task.await;
                settled(&board, code, result, &owner);
            });
        }
    }
}

fn settled(
    board: &PermissionBoard,
    code: PermissionCode,
    result: Result<Finding, JoinError>,
    owner: &Owner,
) {
    board.record(
        code.clone(),
        result.unwrap_or_else(|_| Finding::unknown(Advice::CheckFailed)),
    );
    board.end(&code);
    logged(&view_of(code.clone(), board.get(&code)), owner);
}
