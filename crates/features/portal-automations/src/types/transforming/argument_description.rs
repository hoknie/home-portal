use super::ArgumentType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgumentDescription {
    pub name: &'static str,
    pub argument_type: ArgumentType,
    pub required: bool,
}

impl ArgumentDescription {
    pub const fn required(name: &'static str, argument_type: ArgumentType) -> ArgumentDescription {
        ArgumentDescription {
            name,
            argument_type,
            required: true,
        }
    }

    pub const fn optional(name: &'static str, argument_type: ArgumentType) -> ArgumentDescription {
        ArgumentDescription {
            name,
            argument_type,
            required: false,
        }
    }
}
