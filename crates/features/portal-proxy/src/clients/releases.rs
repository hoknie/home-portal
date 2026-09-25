use std::time::Duration;

use reqwest::{Client, Response, StatusCode};

use crate::types::Release;

#[derive(Debug, Clone)]
pub struct Releases {
    client: Client,
}

impl Releases {
    pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
    pub const READ_TIMEOUT: Duration = Duration::from_secs(10);
    pub const MAXIMUM_BYTES: usize = 100 * 1024 * 1024;
    pub const USER_AGENT: &'static str = "home-portal";

    pub fn new() -> Result<Releases, String> {
        let client = Client::builder()
            .connect_timeout(Self::CONNECT_TIMEOUT)
            .read_timeout(Self::READ_TIMEOUT)
            .user_agent(Self::USER_AGENT)
            .build()
            .map_err(|error| format!("cannot build the download client: {error}"))?;
        Ok(Releases { client })
    }

    pub async fn release(&self, url: &str) -> Result<Release, String> {
        let response = self.get(url).await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(format!("there is no release at {url}"));
        }
        let bytes = self.body(url, response).await?;
        serde_json::from_slice(&bytes).map_err(|error| {
            format!("the release description from {url} is not understood: {error}")
        })
    }

    pub async fn fetch(&self, url: &str) -> Result<Vec<u8>, String> {
        let response = self.get(url).await?;
        self.body(url, response).await
    }

    async fn get(&self, url: &str) -> Result<Response, String> {
        self.client
            .get(url)
            .send()
            .await
            .map_err(|error| format!("cannot download {url}: {error}"))
    }

    async fn body(&self, url: &str, mut response: Response) -> Result<Vec<u8>, String> {
        if !response.status().is_success() {
            return Err(format!("{url} answered {}", response.status()));
        }
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| format!("the download of {url} broke off: {error}"))?
        {
            body.extend_from_slice(&chunk);
            if body.len() > Self::MAXIMUM_BYTES {
                return Err(format!(
                    "{url} is larger than {} MiB",
                    Self::MAXIMUM_BYTES / 1024 / 1024
                ));
            }
        }
        Ok(body)
    }
}
