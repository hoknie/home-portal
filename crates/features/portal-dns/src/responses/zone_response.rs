use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ZoneResponse {
    pub apex: String,
    pub single: bool,
    pub serial: u32,
}
