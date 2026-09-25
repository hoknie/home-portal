use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceResponse {
    pub name: String,
    pub addresses: Vec<String>,
}
