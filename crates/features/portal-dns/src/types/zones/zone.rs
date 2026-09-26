#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Zone {
    pub apex: String,
    pub single: bool,
}

impl Zone {
    pub fn holds(&self, name: &str) -> bool {
        if self.single {
            name == self.apex
        } else {
            crate::helpers::inside(name, &self.apex)
        }
    }
}
