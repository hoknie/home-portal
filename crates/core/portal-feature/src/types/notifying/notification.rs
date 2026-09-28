#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub title: String,
    pub text: String,
}

impl Notification {
    pub fn new(title: impl Into<String>, text: impl Into<String>) -> Notification {
        Notification {
            title: title.into(),
            text: text.into(),
        }
    }

    pub fn message(&self) -> String {
        if self.title.trim().is_empty() {
            return self.text.clone();
        }
        if self.text.trim().is_empty() {
            return self.title.clone();
        }
        format!("{}\n{}", self.title, self.text)
    }
}
