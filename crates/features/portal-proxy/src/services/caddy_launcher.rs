use std::fs::{self, OpenOptions};
use std::io;
use std::process::{Command, Stdio};
use std::thread;

use serde_json::json;

use crate::types::{AdminAddress, CaddyHome};

pub const DATA_VARIABLE: &str = "XDG_DATA_HOME";
pub const CONFIG_VARIABLE: &str = "XDG_CONFIG_HOME";
pub const LOG_BYTES: u64 = 64 * 1024;

pub fn launch(home: &CaddyHome, admin: &AdminAddress) -> io::Result<()> {
    fs::create_dir_all(home.data())?;
    fs::create_dir_all(home.config())?;
    let initial = json!({ "admin": { "listen": admin.listen() } });
    fs::write(home.initial(), initial.to_string())?;
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(home.log())?;
    let mut command = Command::new(home.binary());
    command
        .arg("run")
        .arg("--resume")
        .arg("--config")
        .arg(home.initial())
        .env(DATA_VARIABLE, home.data())
        .env(CONFIG_VARIABLE, home.config())
        .current_dir(&home.directory)
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log);
    detach(&mut command);
    let mut child = command.spawn()?;
    thread::spawn(move || child.wait());
    Ok(())
}

pub fn log_tail(home: &CaddyHome, lines: usize) -> Vec<String> {
    let Ok(bytes) = fs::read(home.log()) else {
        return Vec::new();
    };
    let start = bytes
        .len()
        .saturating_sub(usize::try_from(LOG_BYTES).unwrap_or(usize::MAX));
    let text = String::from_utf8_lossy(&bytes[start..]);
    let all: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    all[all.len().saturating_sub(lines)..]
        .iter()
        .map(|line| line.to_string())
        .collect()
}

#[cfg(unix)]
fn detach(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn detach(_command: &mut Command) {}
