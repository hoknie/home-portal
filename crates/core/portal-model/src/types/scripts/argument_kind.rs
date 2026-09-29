#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgumentKind {
    Text,
    Number,
    Flag,
    Choice(Vec<String>),
}

impl ArgumentKind {
    pub fn name(&self) -> &'static str {
        match self {
            ArgumentKind::Text => "text",
            ArgumentKind::Number => "number",
            ArgumentKind::Flag => "flag",
            ArgumentKind::Choice(_) => "choice",
        }
    }

    pub fn choices(&self) -> &[String] {
        match self {
            ArgumentKind::Choice(choices) => choices,
            _ => &[],
        }
    }

    pub fn accepts(&self, value: &str) -> bool {
        match self {
            ArgumentKind::Text => true,
            ArgumentKind::Number => value.parse::<f64>().is_ok_and(f64::is_finite),
            ArgumentKind::Flag => false,
            ArgumentKind::Choice(choices) => choices.iter().any(|choice| choice == value),
        }
    }
}
