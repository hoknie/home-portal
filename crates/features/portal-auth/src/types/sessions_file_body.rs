use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Session;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionsFileBody {
    #[serde(default)]
    pub sessions: HashMap<String, Session>,
}
