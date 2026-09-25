use std::time::Duration;

use reqwest::{Client, Response};
use serde_json::Value;

use crate::types::{AdminAddress, CaddyProblem};

#[derive(Debug, Clone)]
pub struct CaddyAdmin {
    client: Client,
    base: String,
}

impl CaddyAdmin {
    pub const TIMEOUT: Duration = Duration::from_secs(5);
    pub const LOAD_PATH: &'static str = "/load";
    pub const STOP_PATH: &'static str = "/stop";
    pub const CONFIG_PATH: &'static str = "/config/";
    pub const LOCAL_AUTHORITY_PATH: &'static str = "/pki/ca/local";
    pub const ROOT_CERTIFICATE_FIELD: &'static str = "root_certificate";
    pub const ERROR_FIELD: &'static str = "error";
    pub const SOCKET_BASE: &'static str = "http://localhost";
    pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

    pub fn new(address: &AdminAddress) -> Result<CaddyAdmin, String> {
        let builder = Client::builder().timeout(Self::TIMEOUT).no_proxy();
        let (builder, base) = match address {
            AdminAddress::Http(url) => (builder, url.as_str().trim_end_matches('/').to_string()),
            AdminAddress::Unix(path) => (
                Self::over_socket(builder, path),
                Self::SOCKET_BASE.to_string(),
            ),
        };
        let client = builder
            .build()
            .map_err(|error| format!("cannot build the Caddy admin client: {error}"))?;
        Ok(CaddyAdmin { client, base })
    }

    pub async fn load(&self, configuration: &Value) -> Result<(), CaddyProblem> {
        let response = self
            .client
            .post(format!("{}{}", self.base, Self::LOAD_PATH))
            .json(configuration)
            .send()
            .await
            .map_err(|error| Self::unreachable(&error))?;
        Self::accepted(response).await.map(|_| ())
    }

    pub async fn occupied(address: &AdminAddress) -> bool {
        match address {
            AdminAddress::Http(url) => {
                let target = format!(
                    "{}:{}",
                    url.host_str()
                        .unwrap_or("127.0.0.1")
                        .trim_start_matches('[')
                        .trim_end_matches(']'),
                    url.port_or_known_default().unwrap_or_default()
                );
                tokio::time::timeout(
                    Self::CONNECT_TIMEOUT,
                    tokio::net::TcpStream::connect(target),
                )
                .await
                .is_ok_and(|connection| connection.is_ok())
            }
            AdminAddress::Unix(path) => Self::socket_answers(path).await,
        }
    }

    #[cfg(unix)]
    async fn socket_answers(path: &std::path::Path) -> bool {
        tokio::time::timeout(Self::CONNECT_TIMEOUT, tokio::net::UnixStream::connect(path))
            .await
            .is_ok_and(|connection| connection.is_ok())
    }

    #[cfg(not(unix))]
    async fn socket_answers(_path: &std::path::Path) -> bool {
        false
    }

    pub async fn stop(&self) -> Result<(), CaddyProblem> {
        let response = self
            .client
            .post(format!("{}{}", self.base, Self::STOP_PATH))
            .send()
            .await
            .map_err(|error| Self::unreachable(&error))?;
        Self::accepted(response).await.map(|_| ())
    }

    pub async fn config(&self) -> Result<Value, CaddyProblem> {
        let response = self
            .client
            .get(format!("{}{}", self.base, Self::CONFIG_PATH))
            .send()
            .await
            .map_err(|error| Self::unreachable(&error))?;
        let text = Self::accepted(response).await?;
        serde_json::from_str(&text).map_err(|error| {
            CaddyProblem::Refused(format!(
                "Caddy answered a configuration that is not JSON: {error}"
            ))
        })
    }

    pub async fn root_certificate(&self) -> Result<String, CaddyProblem> {
        let response = self
            .client
            .get(format!("{}{}", self.base, Self::LOCAL_AUTHORITY_PATH))
            .send()
            .await
            .map_err(|error| Self::unreachable(&error))?;
        let text = Self::accepted(response).await?;
        serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|authority| {
                authority
                    .get(Self::ROOT_CERTIFICATE_FIELD)?
                    .as_str()
                    .map(str::to_string)
            })
            .ok_or_else(|| {
                CaddyProblem::Refused("Caddy's local authority has no root certificate".to_string())
            })
    }

    #[cfg(unix)]
    fn over_socket(
        builder: reqwest::ClientBuilder,
        path: &std::path::Path,
    ) -> reqwest::ClientBuilder {
        builder.unix_socket(path.to_path_buf())
    }

    #[cfg(not(unix))]
    fn over_socket(
        builder: reqwest::ClientBuilder,
        _path: &std::path::Path,
    ) -> reqwest::ClientBuilder {
        builder
    }

    fn unreachable(error: &reqwest::Error) -> CaddyProblem {
        CaddyProblem::Unreachable(if error.is_timeout() {
            format!(
                "Caddy's admin API did not answer within {} seconds",
                Self::TIMEOUT.as_secs()
            )
        } else {
            format!("Caddy's admin API cannot be reached: {error}")
        })
    }

    async fn accepted(response: Response) -> Result<String, CaddyProblem> {
        let status = response.status();
        let text = response.text().await.map_err(|error| {
            CaddyProblem::Unreachable(format!("Caddy's answer could not be read: {error}"))
        })?;
        if status.is_success() {
            return Ok(text);
        }
        let message = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|body| body.get(Self::ERROR_FIELD)?.as_str().map(str::to_string))
            .unwrap_or(text);
        Err(CaddyProblem::Refused(format!(
            "Caddy refused with {status}: {}",
            message.trim()
        )))
    }
}
