use std::time::Duration;

use portal_config::SecretString;
use reqwest::Client;
use serde_json::json;

use crate::types::Outgoing;

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

    pub async fn send(&self, token: &SecretString, message: &Outgoing) -> Result<(), String> {
        let address = format!("{}/bot{}/sendMessage", self.endpoint, token.expose());
        let body = json!({ "chat_id": message.chat_id, "text": message.text, "disable_notification": false });
        let mut problem = String::new();
        for attempt in 1..=Self::ATTEMPTS {
            match self.client.post(&address).json(&body).send().await {
                Ok(response) if response.status().is_success() => return Ok(()),
                Ok(response) => problem = format!("telegram answered {}", response.status()),
                Err(error) => problem = error.to_string(),
            }
            if attempt < Self::ATTEMPTS {
                tokio::time::sleep(Self::BACKOFF * attempt).await;
            }
        }
        Err(problem)
    }
}
