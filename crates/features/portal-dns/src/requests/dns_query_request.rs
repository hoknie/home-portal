use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct DnsQueryRequest {
    pub dns: Option<String>,
}
