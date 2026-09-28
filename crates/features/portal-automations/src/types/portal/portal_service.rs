#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PortalService {
    pub id: String,
    pub name: String,
    pub group: Option<String>,
    pub url: String,
    pub address: String,
    pub state: String,
    pub since: Option<String>,
    pub latency_milliseconds: Option<u64>,
    pub public: bool,
}
