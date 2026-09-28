use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TestRequest {
    pub channel: String,
}
