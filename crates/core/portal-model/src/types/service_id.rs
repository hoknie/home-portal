use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceId(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceIdError {
    Empty,
    TooLong,
    MustStartWithLetter,
    InvalidCharacter(char),
}

impl ServiceId {
    pub const MAXIMUM_LENGTH: usize = 63;

    pub fn parse(value: &str) -> Result<ServiceId, ServiceIdError> {
        let first = value.chars().next().ok_or(ServiceIdError::Empty)?;
        if value.len() > Self::MAXIMUM_LENGTH {
            return Err(ServiceIdError::TooLong);
        }
        if !first.is_ascii_lowercase() {
            return Err(ServiceIdError::MustStartWithLetter);
        }
        if let Some(invalid) = value.chars().find(|character| {
            !(character.is_ascii_lowercase() || character.is_ascii_digit() || *character == '-')
        }) {
            return Err(ServiceIdError::InvalidCharacter(invalid));
        }
        Ok(ServiceId(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for ServiceIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceIdError::Empty => formatter.write_str("must not be empty"),
            ServiceIdError::TooLong => write!(
                formatter,
                "must be at most {} characters",
                ServiceId::MAXIMUM_LENGTH
            ),
            ServiceIdError::MustStartWithLetter => {
                formatter.write_str("must start with a lower-case letter")
            }
            ServiceIdError::InvalidCharacter(character) => {
                write!(
                    formatter,
                    "may contain only lower-case letters, digits and hyphens, not {character:?}"
                )
            }
        }
    }
}

impl Serialize for ServiceId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ServiceId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<ServiceId, D::Error> {
        let value = String::deserialize(deserializer)?;
        ServiceId::parse(&value).map_err(serde::de::Error::custom)
    }
}
