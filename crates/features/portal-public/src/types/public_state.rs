use std::sync::Arc;

use crate::ports::{PublicLayout, PublicServices};

#[derive(Clone)]
pub struct PublicState {
    pub services: Arc<dyn PublicServices>,
    pub layout: Arc<dyn PublicLayout>,
}
