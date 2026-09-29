#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unreadable {
    Binary,
    TooLarge,
    Link,
}

impl Unreadable {
    pub fn name(self) -> &'static str {
        match self {
            Unreadable::Binary => "binary",
            Unreadable::TooLarge => "too-large",
            Unreadable::Link => "link",
        }
    }
}
