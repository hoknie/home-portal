use std::env;

use portal_config::configuration_path;

use super::board::FailureBoard;
use super::main_file::MainFile;
use super::problems::problems_of;
use super::recheck::recheck_forever;
use super::router::failure_router;
use crate::boot::{ADDRESS_VARIABLE, Limits, accept_until, bind, located, requested};
use crate::cli::complain;
use crate::types::{BootError, Ended, Restart, Signals};

pub async fn serve(error: BootError, signals: Signals) -> Result<Ended, BootError> {
    complain(&error);
    tracing::error!("the portal cannot start: {error}");
    let location = configuration_path()?;
    let main = MainFile::read(&location.path).listening(env::var(ADDRESS_VARIABLE).ok())?;
    let address = main.address;
    let language = main.language;
    let board = FailureBoard::new(problems_of(&error), main);
    let listener = bind(address).await?;
    tracing::info!(%address, "failure mode: the reason is shown at /fatal/");
    let restart = Restart::default();
    let recheck = tokio::spawn(recheck_forever(
        location.path,
        board.clone(),
        restart.clone(),
    ));
    let router = failure_router(board, located(), language);
    let served = accept_until(
        listener,
        router,
        requested(signals, restart.clone()),
        Limits::SERVED,
    )
    .await;
    recheck.abort();
    served.map_err(|source| BootError::Serve { address, source })?;
    Ok(if restart.requested() {
        Ended::Restart
    } else {
        Ended::Stopped
    })
}
