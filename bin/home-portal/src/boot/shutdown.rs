use crate::types::{Ended, Restart, Signals};

pub async fn requested(signals: Signals, restart: Restart) -> Ended {
    let ended = tokio::select! {
        () = signals.received() => Ended::Stopped,
        () = restart.wanted() => Ended::Restart,
    };
    match ended {
        Ended::Stopped => tracing::info!("shutdown requested"),
        Ended::Restart => tracing::info!("restart requested"),
    }
    ended
}
