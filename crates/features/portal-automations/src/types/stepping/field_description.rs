use super::FieldType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldDescription {
    pub name: &'static str,
    pub field_type: FieldType,
    pub required: bool,
    pub default: Option<&'static str>,
    pub minimum: Option<i64>,
    pub maximum: Option<i64>,
    pub templated: bool,
    pub template_keys: bool,
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
            templated: false,
            template_keys: false,
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

    pub const fn templated(self) -> FieldDescription {
        FieldDescription {
            templated: true,
            ..self
        }
    }

    pub const fn template_keys(self) -> FieldDescription {
        FieldDescription {
            template_keys: true,
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
