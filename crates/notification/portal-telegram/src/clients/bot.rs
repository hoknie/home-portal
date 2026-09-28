use std::time::Duration;

use reqwest::Client;
use serde_json::json;

pub struct Bot {
    client: Client,
    endpoint: String,
}

pub const ENDPOINT: &str = "https://api.telegram.org";

impl Bot {
    pub const TIMEOUT: Duration = Duration::from_secs(10);
    pub const ATTEMPTS: u32 = 3;
    pub const BACKOFF: Duration = Duration::from_secs(2);

    pub fn new(endpoint: &str) -> Result<Bot, String> {
        Client::builder()
            .timeout(Self::TIMEOUT)
            .build()
            .map(|client| Bot {
                client,
                endpoint: endpoint.trim_end_matches('/').to_string(),
            })
            .map_err(|error| error.to_string())
    }

    pub async fn send(&self, token: &str, chat_id: &str, text: &str) -> Result<(), String> {
        let address = format!("{}/bot{token}/sendMessage", self.endpoint);
        let body = json!({ "chat_id": chat_id, "text": text, "disable_notification": false });
        let mut problem = String::new();
        for attempt in 1..=Self::ATTEMPTS {
            match self.client.post(&address).json(&body).send().await {
                Ok(response) if response.status().is_success() => return Ok(()),
                Ok(response) => problem = format!("telegram answered {}", response.status()),
                Err(error) => problem = error.without_url().to_string(),
            }
            if attempt < Self::ATTEMPTS {
                tokio::time::sleep(Self::BACKOFF * attempt).await;
            }
        }
        Err(problem)
    }
}
