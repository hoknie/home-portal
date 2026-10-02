use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetProblem(String);

impl WidgetProblem {
    pub fn new(message: impl Into<String>) -> WidgetProblem {
        WidgetProblem(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WidgetProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
