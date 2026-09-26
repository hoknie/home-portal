use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tokio::sync::watch;

use super::Tail;

#[derive(Debug, Clone)]
pub struct RunControl {
    pub stdout: Arc<Mutex<Tail>>,
    pub stderr: Arc<Mutex<Tail>>,
    pub stop: watch::Receiver<bool>,
    pub grace: Duration,
}

impl RunControl {
    pub const STOP_GRACE: Duration = Duration::from_secs(5);

    pub fn new() -> (watch::Sender<bool>, RunControl) {
        let (sender, receiver) = watch::channel(false);
        let control = RunControl {
            stdout: Arc::new(Mutex::new(Tail::default())),
            stderr: Arc::new(Mutex::new(Tail::default())),
            stop: receiver,
            grace: Self::STOP_GRACE,
        };
        (sender, control)
    }

    pub fn stop_requested(&self) -> bool {
        *self.stop.borrow()
    }

    pub async fn stopped(mut self) {
        if self.stop.wait_for(|stopped| *stopped).await.is_err() {
            std::future::pending::<()>().await;
        }
    }

    pub fn output(&self) -> (Tail, Tail) {
        let read =
            |tail: &Arc<Mutex<Tail>>| tail.lock().unwrap_or_else(PoisonError::into_inner).clone();
        (read(&self.stdout), read(&self.stderr))
    }
}
