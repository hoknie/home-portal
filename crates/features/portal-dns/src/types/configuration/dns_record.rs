use portal_model::Environment;

use crate::types::RecordData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsRecord {
    pub name: String,
    pub data: RecordData,
    pub environments: Option<Vec<Environment>>,
}
