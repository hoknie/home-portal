use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum WidgetHeight {
    #[default]
    Auto,
    Rows(u8),
    Invalid,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Written {
    Number(i64),
    Text(String),
    Other(serde::de::IgnoredAny),
}

impl WidgetHeight {
    pub const AUTO: &'static str = "auto";
    pub const LARGEST: u8 = 8;
    pub const RULE: &'static str = "must be auto or a whole number of rows from 1 to 8";

    pub fn rows(count: i64) -> WidgetHeight {
        match u8::try_from(count) {
            Ok(rows) if (1..=Self::LARGEST).contains(&rows) => WidgetHeight::Rows(rows),
            _ => WidgetHeight::Invalid,
        }
    }

    pub fn is_auto(self) -> bool {
        self == WidgetHeight::Auto
    }
}

impl Serialize for WidgetHeight {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            WidgetHeight::Rows(rows) => serializer.serialize_u8(*rows),
            WidgetHeight::Auto | WidgetHeight::Invalid => serializer.serialize_str(Self::AUTO),
        }
    }
}

impl<'de> Deserialize<'de> for WidgetHeight {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match Written::deserialize(deserializer)? {
            Written::Number(count) => WidgetHeight::rows(count),
            Written::Text(text) if text == Self::AUTO => WidgetHeight::Auto,
            Written::Text(_) | Written::Other(_) => WidgetHeight::Invalid,
        })
    }
}

impl JsonSchema for WidgetHeight {
    fn schema_name() -> Cow<'static, str> {
        "WidgetHeight".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "anyOf": [
                { "type": "string", "const": "auto" },
                { "type": "integer", "minimum": 1, "maximum": 8 }
            ]
        })
    }
}
