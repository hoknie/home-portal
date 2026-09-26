use super::RecordData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRecord {
    pub name: String,
    pub ttl: u32,
    pub data: RecordData,
}
