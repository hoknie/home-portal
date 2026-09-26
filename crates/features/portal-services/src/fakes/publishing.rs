use std::sync::atomic::{AtomicBool, Ordering};

use portal_feature::FieldError;
use portal_model::Publication;

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

    fn problems(&self, _publication: &Publication) -> Vec<FieldError> {
        Vec::new()
    }
}
