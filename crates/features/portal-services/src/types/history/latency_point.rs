use portal_model::ServiceState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatencyPoint {
    pub at: i64,
    pub state: ServiceState,
    pub average: Option<u32>,
    pub minimum: Option<u32>,
    pub maximum: Option<u32>,
}
