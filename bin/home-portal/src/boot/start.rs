use std::env;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use super::run;
use crate::cli::{PASSWORD_HASH, PROBE, PROXY, password_hash, probe, proxy};
use crate::types::Ended;

pub async fn start() -> ExitCode {
    let command = env::args().nth(1);
    match command.as_deref() {
        None => match run().await {
            Ok(Ended::Stopped) => ExitCode::SUCCESS,
            Ok(Ended::Restart) => relaunch(),
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

pub const REPLACED_SUFFIX: &str = " (deleted)";

pub fn relaunch() -> ExitCode {
    let executable = match env::current_exe() {
        Ok(path) => replaced_executable(path),
        Err(error) => {
            tracing::error!(%error, "cannot find the executable to restart");
            eprintln!("home-portal: cannot find the executable to restart: {error}");
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(executable = %executable.display(), "starting afresh");
    let error = Command::new(&executable)
        .args(env::args_os().skip(1))
        .exec();
    tracing::error!(%error, executable = %executable.display(), "cannot restart");
    eprintln!(
        "home-portal: cannot restart {}: {error}",
        executable.display()
    );
    ExitCode::FAILURE
}

pub fn replaced_executable(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    match text.strip_suffix(REPLACED_SUFFIX) {
        Some(original) if Path::new(original).exists() => PathBuf::from(original),
        _ => path,
    }
}
