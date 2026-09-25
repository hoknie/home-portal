use std::fmt;

use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconSource {
    Lucide(String),
    File(String),
    Address(Url),
    Catalog(String),
    Discovered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconSourceError {
    UnknownPrefix(String),
    EmptyValue,
    BadUrl(String),
    BadSlug(String),
}

impl IconSource {
    pub const AUTO: &'static str = "auto";
    pub const LUCIDE: &'static str = "lucide:";
    pub const FILE: &'static str = "file:";
    pub const URL: &'static str = "url:";
    pub const CATALOG: &'static str = "catalog:";

    pub fn parse(value: &str) -> Result<IconSource, IconSourceError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(IconSourceError::EmptyValue);
        }
        if value == Self::AUTO {
            return Ok(IconSource::Discovered);
        }
        if let Some(name) = value.strip_prefix(Self::LUCIDE) {
            return Self::named(name).map(IconSource::Lucide);
        }
        if let Some(path) = value.strip_prefix(Self::FILE) {
            return Self::filled(path).map(IconSource::File);
        }
        if let Some(address) = value.strip_prefix(Self::URL) {
            return Url::parse(address.trim())
                .ok()
                .filter(|url| matches!(url.scheme(), "http" | "https"))
                .map(IconSource::Address)
                .ok_or_else(|| IconSourceError::BadUrl(address.trim().to_string()));
        }
        if let Some(slug) = value.strip_prefix(Self::CATALOG) {
            let slug = slug.trim();
            let plain = !slug.is_empty()
                && slug.chars().all(|character| {
                    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
                });
            return plain
                .then(|| IconSource::Catalog(slug.to_string()))
                .ok_or_else(|| IconSourceError::BadSlug(slug.to_string()));
        }
        if value.contains(':') {
            let prefix = value.split(':').next().unwrap_or_default().to_string();
            return Err(IconSourceError::UnknownPrefix(prefix));
        }
        Self::named(value).map(IconSource::Lucide)
    }

    pub fn is_fetched(&self) -> bool {
        matches!(
            self,
            IconSource::Address(_)
                | IconSource::Catalog(_)
                | IconSource::Discovered
                | IconSource::File(_)
        )
    }

    fn named(name: &str) -> Result<String, IconSourceError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(IconSourceError::EmptyValue);
        }
        Ok(name.to_string())
    }

    fn filled(path: &str) -> Result<String, IconSourceError> {
        let path = path.trim();
        if path.is_empty() {
            return Err(IconSourceError::EmptyValue);
        }
        Ok(path.to_string())
    }
}

impl fmt::Display for IconSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IconSourceError::UnknownPrefix(prefix) => write!(
                formatter,
                "{prefix}: is not a way to name an icon; use lucide:, file:, url:, catalog: or auto"
            ),
            IconSourceError::EmptyValue => formatter.write_str("must not be empty"),
            IconSourceError::BadUrl(value) => {
                write!(formatter, "{value} is not an absolute http or https URL")
            }
            IconSourceError::BadSlug(value) => write!(
                formatter,
                "{value} is not a catalogue name; use lower-case letters, digits and hyphens"
            ),
        }
    }
}
