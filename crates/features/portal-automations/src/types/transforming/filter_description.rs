use super::{ArgumentDescription, ValueType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterDescription {
    pub name: &'static str,
    pub accepts: &'static [ValueType],
    pub gives: ValueType,
    pub element: bool,
    pub arguments: &'static [ArgumentDescription],
}

impl FilterDescription {
    pub fn fewest_arguments(&self) -> usize {
        self.arguments
            .iter()
            .filter(|argument| argument.required)
            .count()
    }

    pub fn takes(&self, value_type: ValueType) -> bool {
        self.accepts.contains(&ValueType::Any) || self.accepts.contains(&value_type)
    }

    pub fn accepted(&self) -> String {
        let names: Vec<&str> = self
            .accepts
            .iter()
            .map(|value_type| value_type.described())
            .collect();
        names.join(" or ")
    }
}
