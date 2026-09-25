use std::fmt;
use std::net::IpAddr;
use std::path::PathBuf;

use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminAddress {
    Http(Url),
    Unix(PathBuf),
}

impl AdminAddress {
    pub const DEFAULT: &'static str = "http://127.0.0.1:2019";
    pub const UNIX_PREFIX: &'static str = "unix:";
    pub const LOCALHOST: &'static str = "localhost";
    pub const PROBLEM: &'static str = "must be an http URL on a loopback address, such as http://127.0.0.1:2019, or unix:<absolute path>";

    pub fn parse(text: &str) -> Result<AdminAddress, &'static str> {
        let text = text.trim();
        if let Some(path) = text.strip_prefix(Self::UNIX_PREFIX) {
            let path = PathBuf::from(path);
            return if path.is_absolute() {
                Ok(AdminAddress::Unix(path))
            } else {
                Err(Self::PROBLEM)
            };
        }
        let url = Url::parse(text).map_err(|_| Self::PROBLEM)?;
        if url.scheme() != "http" || !Self::loopback(&url) {
            return Err(Self::PROBLEM);
        }
        Ok(AdminAddress::Http(url))
    }

    pub fn listen(&self) -> String {
        match self {
            AdminAddress::Http(url) => format!(
                "{}:{}",
                url.host_str().unwrap_or(Self::LOCALHOST),
                url.port_or_known_default().unwrap_or_default()
            ),
            AdminAddress::Unix(path) => format!("unix/{}", path.display()),
        }
    }

    pub fn loopback(url: &Url) -> bool {
        match url.host_str() {
            Some(Self::LOCALHOST) => true,
            Some(host) => host
                .trim_start_matches('[')
                .trim_end_matches(']')
                .parse::<IpAddr>()
                .is_ok_and(|address| address.is_loopback()),
            None => false,
        }
    }
}

impl Default for AdminAddress {
    fn default() -> AdminAddress {
        AdminAddress::parse(Self::DEFAULT).unwrap_or_else(|_| AdminAddress::Unix(PathBuf::new()))
    }
}

impl fmt::Display for AdminAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdminAddress::Http(url) => formatter.write_str(url.as_str().trim_end_matches('/')),
            AdminAddress::Unix(path) => {
                write!(formatter, "{}{}", Self::UNIX_PREFIX, path.display())
            }
        }
    }
}
