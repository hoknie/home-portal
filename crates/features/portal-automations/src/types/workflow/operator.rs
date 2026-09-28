#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Contains,
    IsEmpty,
    IsNotEmpty,
}

impl Operator {
    pub const ALL: [Operator; 9] = [
        Operator::Equal,
        Operator::NotEqual,
        Operator::Less,
        Operator::LessOrEqual,
        Operator::Greater,
        Operator::GreaterOrEqual,
        Operator::Contains,
        Operator::IsEmpty,
        Operator::IsNotEmpty,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Operator::Equal => "==",
            Operator::NotEqual => "!=",
            Operator::Less => "<",
            Operator::LessOrEqual => "<=",
            Operator::Greater => ">",
            Operator::GreaterOrEqual => ">=",
            Operator::Contains => "contains",
            Operator::IsEmpty => "is-empty",
            Operator::IsNotEmpty => "is-not-empty",
        }
    }

    pub fn of(name: &str) -> Option<Operator> {
        Self::ALL
            .into_iter()
            .find(|operator| operator.name() == name)
    }

    pub fn takes_right(self) -> bool {
        !matches!(self, Operator::IsEmpty | Operator::IsNotEmpty)
    }
}
