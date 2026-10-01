use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Http {
        status: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
        delay: Duration,
    },
    Raw(Vec<u8>),
    Hang,
}

impl Answer {
    pub fn status(status: u16) -> Answer {
        Answer::Http {
            status,
            headers: Vec::new(),
            body: Vec::new(),
            delay: Duration::ZERO,
        }
    }

    pub fn ok(content_type: &str, body: impl Into<Vec<u8>>) -> Answer {
        Answer::status(200)
            .with_header("content-type", content_type)
            .with_body(body)
    }

    pub fn json(body: &str) -> Answer {
        Answer::ok("application/json", body)
    }

    pub fn with_body(self, body: impl Into<Vec<u8>>) -> Answer {
        match self {
            Answer::Http {
                status,
                headers,
                delay,
                ..
            } => Answer::Http {
                status,
                headers,
                body: body.into(),
                delay,
            },
            other => other,
        }
    }

    pub fn with_header(self, name: &str, value: &str) -> Answer {
        match self {
            Answer::Http {
                status,
                mut headers,
                body,
                delay,
            } => {
                headers.push((name.to_string(), value.to_string()));
                Answer::Http {
                    status,
                    headers,
                    body,
                    delay,
                }
            }
            other => other,
        }
    }

    pub fn after(self, wait: Duration) -> Answer {
        match self {
            Answer::Http {
                status,
                headers,
                body,
                ..
            } => Answer::Http {
                status,
                headers,
                body,
                delay: wait,
            },
            other => other,
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Answer::Http {
                status,
                headers,
                body,
                ..
            } => {
                let mut head = format!("HTTP/1.1 {status} X\r\n");
                for (name, value) in headers {
                    head.push_str(&format!("{name}: {value}\r\n"));
                }
                head.push_str(&format!(
                    "content-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                ));
                let mut bytes = head.into_bytes();
                bytes.extend_from_slice(body);
                bytes
            }
            Answer::Raw(bytes) => bytes.clone(),
            Answer::Hang => Vec::new(),
        }
    }
}
