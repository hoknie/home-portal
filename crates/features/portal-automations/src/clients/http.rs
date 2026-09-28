use reqwest::header::{CONTENT_TYPE, HeaderName, HeaderValue};
use reqwest::redirect::Policy;
use reqwest::{Client, Method, Url};

use crate::types::{HttpAnswer, HttpRequest};

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub const JSON: &'static str = "application/json";

    pub fn new() -> Result<HttpClient, String> {
        let client = Client::builder()
            .redirect(Policy::limited(HttpAnswer::MOST_REDIRECTS))
            .build()
            .map_err(|error| error.to_string())?;
        Ok(HttpClient { client })
    }

    pub async fn send(&self, request: &HttpRequest) -> Result<HttpAnswer, String> {
        let url =
            Url::parse(request.url.trim()).map_err(|error| format!("{}: {error}", request.url))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(format!("{url} is not an http or https address"));
        }
        let method =
            Method::from_bytes(request.method.as_bytes()).map_err(|error| error.to_string())?;
        let mut builder = self.client.request(method, url).timeout(request.timeout);
        let mut typed = false;
        for (name, value) in &request.headers {
            let name =
                HeaderName::from_bytes(name.as_bytes()).map_err(|error| error.to_string())?;
            let value = HeaderValue::from_str(value).map_err(|error| format!("{name}: {error}"))?;
            typed |= name == CONTENT_TYPE;
            builder = builder.header(name, value);
        }
        if let Some(body) = &request.body {
            if !typed && serde_json::from_str::<serde_json::Value>(body).is_ok() {
                builder = builder.header(CONTENT_TYPE, Self::JSON);
            }
            builder = builder.body(body.clone());
        }
        let mut response = builder.send().await.map_err(|error| describe(&error))?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_string(),
                    String::from_utf8_lossy(value.as_bytes()).to_string(),
                )
            })
            .collect();
        let mut kept = Vec::new();
        let mut read = 0usize;
        while let Some(chunk) = response.chunk().await.map_err(|error| describe(&error))? {
            read += chunk.len();
            if kept.len() < HttpAnswer::LARGEST_BODY {
                let room = HttpAnswer::LARGEST_BODY - kept.len();
                kept.extend_from_slice(&chunk[..chunk.len().min(room)]);
            }
            if read >= HttpAnswer::LARGEST_READ {
                break;
            }
        }
        let body = String::from_utf8_lossy(&kept).to_string();
        let json = serde_json::from_str(&body).ok();
        Ok(HttpAnswer {
            status,
            headers,
            body,
            json,
        })
    }
}

fn describe(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        return "the request timed out".to_string();
    }
    if error.is_redirect() {
        return format!("more than {} redirects", HttpAnswer::MOST_REDIRECTS);
    }
    let mut text = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(inner) = source {
        text.push_str(": ");
        text.push_str(&inner.to_string());
        source = inner.source();
    }
    text
}
