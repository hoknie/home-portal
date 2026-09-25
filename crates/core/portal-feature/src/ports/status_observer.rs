use crate::types::StatusChange;

pub trait StatusObserver: Send + Sync {
    fn changed(&self, change: &StatusChange);
}
