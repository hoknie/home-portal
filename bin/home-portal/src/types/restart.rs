use std::sync::Arc;

use tokio::sync::watch;

#[derive(Debug, Clone)]
pub struct Restart {
    wanted: Arc<watch::Sender<bool>>,
}

impl Default for Restart {
    fn default() -> Restart {
        Restart {
            wanted: Arc::new(watch::channel(false).0),
        }
    }
}

impl Restart {
    pub fn request(&self) {
        self.wanted.send_replace(true);
    }

    pub fn requested(&self) -> bool {
        *self.wanted.borrow()
    }

    pub async fn wanted(&self) {
        let mut receiver = self.wanted.subscribe();
        let _ = receiver.wait_for(|wanted| *wanted).await;
    }
}
