#[derive(Debug, Clone, Default)]
pub struct DnsHttpsChoice {
    pub enabled: bool,
    pub host: Option<String>,
}
