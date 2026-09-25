use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Environment(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentError {
    Empty,
    TooLong,
    MustStartWithLetter,
    InvalidCharacter(char),
}

impl Environment {
    pub const INTERNET: &'static str = "internet";
    pub const MAXIMUM_LENGTH: usize = 32;

    pub fn internet() -> Environment {
        Environment(Self::INTERNET.to_string())
    }

    pub fn parse(value: &str) -> Result<Environment, EnvironmentError> {
        let first = value.chars().next().ok_or(EnvironmentError::Empty)?;
        if value.len() > Self::MAXIMUM_LENGTH {
            return Err(EnvironmentError::TooLong);
        }
        if !first.is_ascii_lowercase() {
            return Err(EnvironmentError::MustStartWithLetter);
        }
        if let Some(invalid) = value.chars().find(|character| {
            !(character.is_ascii_lowercase() || character.is_ascii_digit() || *character == '-')
        }) {
            return Err(EnvironmentError::InvalidCharacter(invalid));
        }
        Ok(Environment(value.to_string()))
    }

    pub fn is_internet(&self) -> bool {
        self.0 == Self::INTERNET
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvironmentError::Empty => formatter.write_str("must not be empty"),
            EnvironmentError::TooLong => write!(
                formatter,
                "must be at most {} characters",
                Environment::MAXIMUM_LENGTH
            ),
            EnvironmentError::MustStartWithLetter => {
                formatter.write_str("must start with a lower-case letter")
            }
            EnvironmentError::InvalidCharacter(character) => write!(
                formatter,
                "may contain only lower-case letters, digits and hyphens, not {character:?}"
            ),
        }
    }
}

impl Serialize for Environment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Environment, D::Error> {
        let value = String::deserialize(deserializer)?;
        Environment::parse(&value).map_err(serde::de::Error::custom)
    }
}
