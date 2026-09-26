use std::future;

#[cfg(unix)]
use tokio::signal::unix::{Signal, SignalKind, signal};

#[derive(Debug)]
pub struct Signals {
    #[cfg(unix)]
    interrupt: Option<Signal>,
    #[cfg(unix)]
    terminate: Option<Signal>,
}

impl Signals {
    #[cfg(unix)]
    pub fn install() -> Signals {
        Signals {
            interrupt: signal(SignalKind::interrupt()).ok(),
            terminate: signal(SignalKind::terminate()).ok(),
        }
    }

    #[cfg(not(unix))]
    pub fn install() -> Signals {
        Signals {}
    }

    #[cfg(unix)]
    pub async fn received(self) {
        tokio::select! {
            () = arrival(self.interrupt) => {}
            () = arrival(self.terminate) => {}
        }
    }

    #[cfg(not(unix))]
    pub async fn received(self) {
        if tokio::signal::ctrl_c().await.is_err() {
            future::pending::<()>().await;
        }
    }
}

#[cfg(unix)]
async fn arrival(stream: Option<Signal>) {
    match stream {
        Some(mut stream) => {
            stream.recv().await;
        }
        None => future::pending::<()>().await,
    }
}
