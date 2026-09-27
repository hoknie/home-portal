use std::env;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::Parser;

use super::run;
use crate::cli::{fail, password_hash, probe, proxy};
use crate::types::{Command as Invocation, CommandLine, Ended};

pub async fn start() -> ExitCode {
    let line = match CommandLine::try_parse() {
        Ok(line) => line,
        Err(error) => {
            let _ = error.print();
            return ExitCode::from(u8::try_from(error.exit_code()).unwrap_or(u8::MAX));
        }
    };
    match line.command.unwrap_or(Invocation::Serve) {
        Invocation::Serve => serve().await,
        Invocation::PasswordHash => password_hash(),
        Invocation::Probe { target, kind } => probe(target, kind.map(Into::into)).await,
        Invocation::Proxy { action } => proxy(action),
    }
}

async fn serve() -> ExitCode {
    match run().await {
        Ok(Ended::Stopped) => ExitCode::SUCCESS,
        Ok(Ended::Restart) => relaunch(),
        Err(error) => fail(error),
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
