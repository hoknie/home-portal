#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CookieScope {
    pub secure: bool,
    pub domain: Option<String>,
}
