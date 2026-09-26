use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process::{ExitStatus, Stdio};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;

use super::{kill_group, terminate_group};
use crate::types::{Finished, Invocation, Outcome, RunControl, Tail};

pub trait GroupRegistry: Send + Sync {
    fn started(&self, group: u32);
    fn reaped(&self, group: u32);
}

pub struct Runner;

enum Ended {
    Exited(Option<ExitStatus>),
    TimedOut,
    Stopped(Option<ExitStatus>),
}

impl Runner {
    pub const DRAIN_GRACE: Duration = Duration::from_secs(1);
    const CHUNK: usize = 8192;
    const BUSY_ATTEMPTS: u32 = 20;
    const BUSY_PAUSE: Duration = Duration::from_millis(50);

    pub async fn run(
        invocation: &Invocation,
        groups: Arc<dyn GroupRegistry>,
        control: RunControl,
    ) -> Finished {
        let began = Instant::now();
        if control.stop_requested() {
            return Self::finished(Outcome::Stopped, (None, None), began, control.output());
        }
        let mut command = Command::new(&invocation.program);
        command
            .args(&invocation.arguments)
            .env_clear()
            .envs(invocation.environment.iter().cloned())
            .current_dir(&invocation.directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true);
        let mut child = match Self::spawn(&mut command).await {
            Ok(child) => child,
            Err(error) => {
                return Self::finished(
                    Outcome::Failed,
                    (None, Some(format!("the script could not start: {error}"))),
                    began,
                    (Tail::default(), Tail::default()),
                );
            }
        };
        let group = child.id().unwrap_or_default();
        groups.started(group);
        let stdout = child
            .stdout
            .take()
            .map(|stream| Self::drain(stream, control.stdout.clone()));
        let stderr = child
            .stderr
            .take()
            .map(|stream| Self::drain(stream, control.stderr.clone()));
        if let Some(mut input) = child.stdin.take() {
            let text = invocation.input.clone();
            tokio::spawn(async move {
                let _ = input.write_all(text.as_bytes()).await;
            });
        }
        let ended = Self::wait(&mut child, group, invocation.timeout, control).await;
        kill_group(group);
        let ended = match ended {
            Ended::TimedOut => {
                Self::reap(&mut child).await;
                Ended::TimedOut
            }
            Ended::Stopped(None) => Ended::Stopped(Self::reap(&mut child).await),
            other => other,
        };
        groups.reaped(group);
        let tails = (Self::collect(stdout).await, Self::collect(stderr).await);
        let (outcome, detail) = match ended {
            Ended::TimedOut => (
                Outcome::TimedOut,
                (
                    None,
                    Some(format!(
                        "killed after {} seconds",
                        invocation.timeout.as_secs()
                    )),
                ),
            ),
            Ended::Stopped(status) => (
                Outcome::Stopped,
                (status.and_then(|status| status.code()), None),
            ),
            Ended::Exited(Some(status)) => Self::judged(status),
            Ended::Exited(None) => (
                Outcome::Failed,
                (
                    None,
                    Some("the script's exit could not be read".to_string()),
                ),
            ),
        };
        Self::finished(outcome, detail, began, tails)
    }

    async fn wait(child: &mut Child, group: u32, timeout: Duration, control: RunControl) -> Ended {
        let grace = control.grace;
        tokio::select! {
            status = child.wait() => Ended::Exited(status.ok()),
            _ = tokio::time::sleep(timeout) => Ended::TimedOut,
            _ = control.stopped() => {
                terminate_group(group);
                match tokio::time::timeout(grace, child.wait()).await {
                    Ok(status) => Ended::Stopped(status.ok()),
                    Err(_) => Ended::Stopped(None),
                }
            }
        }
    }

    async fn spawn(command: &mut Command) -> io::Result<Child> {
        let mut attempt = 0;
        loop {
            match command.spawn() {
                Err(error)
                    if error.kind() == io::ErrorKind::ExecutableFileBusy
                        && attempt < Self::BUSY_ATTEMPTS =>
                {
                    attempt += 1;
                    tokio::time::sleep(Self::BUSY_PAUSE).await;
                }
                result => return result,
            }
        }
    }

    fn judged(status: ExitStatus) -> (Outcome, (Option<i32>, Option<String>)) {
        match (status.code(), status.signal()) {
            (Some(0), _) => (Outcome::Succeeded, (Some(0), None)),
            (Some(code), _) => (Outcome::Failed, (Some(code), None)),
            (None, Some(signal)) => (
                Outcome::Failed,
                (None, Some(format!("killed by signal {signal}"))),
            ),
            (None, None) => (Outcome::Failed, (None, None)),
        }
    }

    async fn reap(child: &mut Child) -> Option<ExitStatus> {
        let _ = child.start_kill();
        child.wait().await.ok()
    }

    fn drain(
        mut stream: impl AsyncRead + Unpin + Send + 'static,
        tail: Arc<Mutex<Tail>>,
    ) -> (Arc<Mutex<Tail>>, JoinHandle<()>) {
        let shared = tail.clone();
        let reader = tokio::spawn(async move {
            let mut buffer = vec![0u8; Self::CHUNK];
            while let Ok(read) = stream.read(&mut buffer).await {
                if read == 0 {
                    break;
                }
                shared
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(&buffer[..read]);
            }
        });
        (tail, reader)
    }

    async fn collect(reader: Option<(Arc<Mutex<Tail>>, JoinHandle<()>)>) -> Tail {
        let Some((tail, mut reader)) = reader else {
            return Tail::default();
        };
        if tokio::time::timeout(Self::DRAIN_GRACE, &mut reader)
            .await
            .is_err()
        {
            reader.abort();
        }
        tail.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    fn finished(
        outcome: Outcome,
        detail: (Option<i32>, Option<String>),
        began: Instant,
        tails: (Tail, Tail),
    ) -> Finished {
        Finished {
            outcome,
            exit_code: detail.0,
            reason: detail.1,
            duration: began.elapsed(),
            stdout: tails.0,
            stderr: tails.1,
        }
    }
}
