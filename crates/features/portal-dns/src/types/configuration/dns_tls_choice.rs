#[derive(Debug, Clone, Default)]
pub struct DnsTlsChoice {
    pub enabled: bool,
    pub port: Option<u16>,
    pub certificate: Option<String>,
    pub key: Option<String>,
}
