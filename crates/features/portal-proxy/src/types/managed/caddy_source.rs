use std::fmt;

use url::Url;

use crate::types::AdminAddress;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CaddyVersion {
    #[default]
    Latest,
    Exact(String),
}

impl CaddyVersion {
    pub const LATEST: &'static str = "latest";
    pub const PROBLEM: &'static str =
        "must be latest or an exact version such as 2.10.2 or 2.11.0-beta.1";

    pub fn parse(text: &str) -> Result<CaddyVersion, &'static str> {
        let text = text.trim();
        if text == Self::LATEST {
            return Ok(CaddyVersion::Latest);
        }
        let version = text.strip_prefix('v').unwrap_or(text);
        let (release, suffix) = match version.split_once('-') {
            Some((release, suffix)) => (release, Some(suffix)),
            None => (version, None),
        };
        let numbers = release.split('.').collect::<Vec<_>>();
        let numeric = numbers.len() == 3
            && numbers
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
        let suffix_valid = suffix.is_none_or(|suffix| {
            !suffix.is_empty()
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-')
        });
        if numeric && suffix_valid {
            Ok(CaddyVersion::Exact(version.to_string()))
        } else {
            Err(Self::PROBLEM)
        }
    }
}

impl fmt::Display for CaddyVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaddyVersion::Latest => formatter.write_str(Self::LATEST),
            CaddyVersion::Exact(version) => formatter.write_str(version),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaddySource {
    pub base: String,
    pub version: CaddyVersion,
}

impl CaddySource {
    pub const DEFAULT_BASE: &'static str =
        "https://api.github.com/repos/caddyserver/caddy/releases";
    pub const PROBLEM: &'static str = "must be an https URL of a GitHub-compatible releases API, or an http URL on a loopback address";

    pub fn parse_base(text: &str) -> Result<String, &'static str> {
        let base = text.trim().trim_end_matches('/');
        let url = Url::parse(base).map_err(|_| Self::PROBLEM)?;
        let allowed = match url.scheme() {
            "https" => url.host_str().is_some(),
            "http" => AdminAddress::loopback(&url),
            _ => false,
        };
        if allowed {
            Ok(base.to_string())
        } else {
            Err(Self::PROBLEM)
        }
    }

    pub fn release_url(&self) -> String {
        match &self.version {
            CaddyVersion::Latest => format!("{}/latest", self.base),
            CaddyVersion::Exact(version) => format!("{}/tags/v{version}", self.base),
        }
    }

    pub fn is_default_base(&self) -> bool {
        self.base == Self::DEFAULT_BASE
    }
}

impl Default for CaddySource {
    fn default() -> CaddySource {
        CaddySource {
            base: Self::DEFAULT_BASE.to_string(),
            version: CaddyVersion::Latest,
        }
    }
}
