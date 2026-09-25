use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::{Client, Url};

use super::Fetched;

pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
    pub const TIMEOUT: Duration = Duration::from_secs(10);
    pub const REDIRECTS: usize = 3;
    pub const ICON_CEILING_BYTES: usize = 512 * 1024;
    pub const PAGE_CEILING_BYTES: usize = 512 * 1024;

    pub fn new() -> Result<Fetcher, String> {
        let client = Client::builder()
            .connect_timeout(Self::CONNECT_TIMEOUT)
            .timeout(Self::TIMEOUT)
            .redirect(Policy::custom(|attempt| {
                let same_origin = attempt
                    .previous()
                    .first()
                    .map(|first| first.origin() == attempt.url().origin())
                    .unwrap_or(true);
                if attempt.previous().len() > Self::REDIRECTS {
                    attempt.error("too many redirects")
                } else if same_origin {
                    attempt.follow()
                } else {
                    attempt.error("a redirect left the service's own address")
                }
            }))
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Fetcher { client })
    }

    pub async fn get(&self, url: &Url, ceiling: usize) -> Result<Fetched, String> {
        let mut response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(|error| error.to_string())?;
        if !response.status().is_success() {
            return Err(format!("answered {}", response.status()));
        }
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
            if bytes.len() + chunk.len() > ceiling {
                return Err(format!(
                    "is larger than the {} KiB the portal reads",
                    ceiling / 1024
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(Fetched {
            bytes,
            content_type,
        })
    }
}
