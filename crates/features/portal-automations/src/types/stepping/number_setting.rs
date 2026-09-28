#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberSetting {
    Fixed(u64),
    Template(String),
}

impl NumberSetting {
    pub fn template(&self) -> Option<&str> {
        match self {
            NumberSetting::Fixed(_) => None,
            NumberSetting::Template(template) => Some(template),
        }
    }
}
