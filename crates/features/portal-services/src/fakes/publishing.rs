use std::sync::atomic::{AtomicBool, Ordering};

use crate::ports::Publishing;

#[derive(Debug, Default)]
pub struct Switch(pub AtomicBool);

impl Switch {
    pub fn on() -> Switch {
        Switch(AtomicBool::new(true))
    }
}

impl Publishing for Switch {
    fn https_port(&self) -> Option<u16> {
        self.0.load(Ordering::Relaxed).then_some(443)
    }
}
