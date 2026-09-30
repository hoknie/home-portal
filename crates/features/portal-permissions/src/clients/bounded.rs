use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::types::Ran;

pub const POLL: Duration = Duration::from_millis(50);

pub fn run_bounded(mut command: Command, limit: Duration) -> Ran {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => return Ran::NotStarted,
    };
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_or(Ran::NotStarted, Ran::Finished);
            }
            Ok(None) if started.elapsed() >= limit => {
                let _ = child.kill();
                let _ = child.wait();
                return Ran::TimedOut;
            }
            Ok(None) => thread::sleep(POLL),
            Err(_) => return Ran::NotStarted,
        }
    }
}
