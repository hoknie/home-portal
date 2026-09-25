use std::time::Duration;

use portal_config::SecretString;
use portal_feature::WidgetProblem;
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use url::Url;

pub struct FeedClient {
    client: Client,
}

impl FeedClient {
    pub const TIMEOUT: Duration = Duration::from_secs(15);
    pub const CEILING_BYTES: usize = 5 * 1024 * 1024;

    pub fn new() -> Result<FeedClient, String> {
        Client::builder()
            .timeout(Self::TIMEOUT)
            .build()
            .map(|client| FeedClient { client })
            .map_err(|error| error.to_string())
    }

    pub async fn fetch(
        &self,
        url: &Url,
        secret: Option<&SecretString>,
    ) -> Result<String, WidgetProblem> {
        let mut request = self.client.get(url.clone());
        if let Some(secret) = secret {
            request = match url.username() {
                "" => request.header(AUTHORIZATION, format!("Bearer {}", secret.expose())),
                user => request.basic_auth(user, Some(secret.expose())),
            };
        }
        let mut response = request.send().await.map_err(|error| {
            WidgetProblem::new(format!("the calendar could not be fetched: {error}"))
        })?;
        if !response.status().is_success() {
            return Err(WidgetProblem::new(format!(
                "the calendar answered {}",
                response.status()
            )));
        }
        let mut text = String::new();
        while let Some(chunk) = response.chunk().await.map_err(|error| {
            WidgetProblem::new(format!("the calendar could not be read: {error}"))
        })? {
            if text.len() + chunk.len() > Self::CEILING_BYTES {
                return Err(WidgetProblem::new(format!(
                    "the calendar is larger than the {} MiB the portal reads",
                    Self::CEILING_BYTES / 1024 / 1024
                )));
            }
            text.push_str(&String::from_utf8_lossy(&chunk));
        }
        Ok(text)
    }
}
