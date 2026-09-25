use std::sync::Arc;

use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::types::ServiceEntry;

pub struct Running {
    pub entry: ServiceEntry,
    pub handle: JoinHandle<()>,
    pub wake: Arc<Notify>,
}
