#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Visitor {
    pub user: String,
    pub address: String,
    pub environment: String,
    pub reason: Option<&'static str>,
}
