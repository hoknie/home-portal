#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentType {
    Text,
    Number,
    Any,
    Condition,
    Template,
    Order,
    Operations,
}

impl ArgumentType {
    pub const ORDERS: &'static [&'static str] = &["asc", "desc"];

    pub fn name(self) -> &'static str {
        match self {
            ArgumentType::Text => "text",
            ArgumentType::Number => "number",
            ArgumentType::Any => "any",
            ArgumentType::Condition => "condition",
            ArgumentType::Template => "template",
            ArgumentType::Order => "order",
            ArgumentType::Operations => "operations",
        }
    }
}
