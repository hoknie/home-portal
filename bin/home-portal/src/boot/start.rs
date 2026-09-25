use std::env;
use std::process::ExitCode;

use super::run;
use crate::cli::{PASSWORD_HASH, PROBE, PROXY, password_hash, probe, proxy};

pub async fn start() -> ExitCode {
    let command = env::args().nth(1);
    match command.as_deref() {
        None => match run().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("home-portal: {error}");
                ExitCode::FAILURE
            }
        },
        Some(PASSWORD_HASH) => password_hash(),
        Some(PROBE) => probe(&env::args().skip(2).collect::<Vec<_>>()).await,
        Some(PROXY) => proxy(&env::args().skip(2).collect::<Vec<_>>()),
        Some(other) => {
            eprintln!(
                "home-portal: unknown command {other:?}; the commands are {PASSWORD_HASH}, {PROBE} and {PROXY}"
            );
            ExitCode::FAILURE
        }
    }
}
