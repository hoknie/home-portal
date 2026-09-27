use std::fmt::Display;
use std::io::Write;
use std::process::ExitCode;

use super::palette::ERROR;

pub fn fail(message: impl Display) -> ExitCode {
    let _ = writeln!(anstream::stderr(), "{}", failure_line(message));
    ExitCode::FAILURE
}

pub fn failure_line(message: impl Display) -> String {
    format!("{ERROR}error:{ERROR:#} {message}")
}
