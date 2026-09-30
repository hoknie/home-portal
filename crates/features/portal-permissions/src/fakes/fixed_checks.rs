use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::DelayedCheck;
use crate::ports::{CheckSource, PermissionCheck};
use crate::services::codes_of;
use crate::types::{Advice, Finding, PermissionCode, PermissionSettings};

pub struct FixedChecks {
    pub applies: bool,
    pub delay: Duration,
    pub denied: Vec<PermissionCode>,
    pub asked: Arc<AtomicUsize>,
}

impl FixedChecks {
    pub fn granting(delay: Duration) -> FixedChecks {
        FixedChecks {
            applies: true,
            delay,
            denied: Vec::new(),
            asked: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl CheckSource for FixedChecks {
    fn applies(&self) -> bool {
        self.applies
    }

    fn checks(&self, settings: &PermissionSettings) -> Vec<Arc<dyn PermissionCheck>> {
        self.asked.fetch_add(1, Ordering::SeqCst);
        codes_of(settings)
            .into_iter()
            .map(|code| {
                let finding = if self.denied.contains(&code) {
                    Finding::denied(Advice::AllowInSettings)
                } else {
                    Finding::granted()
                };
                Arc::new(DelayedCheck {
                    code,
                    finding,
                    delay: self.delay,
                }) as Arc<dyn PermissionCheck>
            })
            .collect()
    }
}
