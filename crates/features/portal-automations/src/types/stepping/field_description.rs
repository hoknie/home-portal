use super::FieldType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldDescription {
    pub name: &'static str,
    pub field_type: FieldType,
    pub required: bool,
    pub default: Option<&'static str>,
    pub minimum: Option<i64>,
    pub maximum: Option<i64>,
}

impl FieldDescription {
    pub const fn of(name: &'static str, field_type: FieldType) -> FieldDescription {
        FieldDescription {
            name,
            field_type,
            required: false,
            default: None,
            minimum: None,
            maximum: None,
        }
    }

    pub const fn required(self) -> FieldDescription {
        FieldDescription {
            required: true,
            ..self
        }
    }

    pub const fn defaulting(self, default: &'static str) -> FieldDescription {
        FieldDescription {
            default: Some(default),
            ..self
        }
    }

    pub const fn between(self, minimum: i64, maximum: i64) -> FieldDescription {
        FieldDescription {
            minimum: Some(minimum),
            maximum: Some(maximum),
            ..self
        }
    }
}
