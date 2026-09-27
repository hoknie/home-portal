use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SwitchRequest {
    pub enabled: bool,
}
