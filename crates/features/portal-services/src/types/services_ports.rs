use std::sync::Arc;

use portal_feature::{EventSink, StatusObserver};

use crate::ports::Publishing;

pub struct ServicesPorts {
    pub observers: Vec<Arc<dyn StatusObserver>>,
    pub publishing: Arc<dyn Publishing>,
    pub events: Arc<dyn EventSink>,
}
