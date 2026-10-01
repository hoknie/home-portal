#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub head: String,
    pub body: String,
}

impl Request {
    pub fn parse(bytes: &[u8]) -> Request {
        let text = String::from_utf8_lossy(bytes).to_string();
        let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
        let mut first = head.lines().next().unwrap_or_default().split_whitespace();
        Request {
            method: first.next().unwrap_or_default().to_string(),
            path: first.next().unwrap_or("/").to_string(),
            head: head.to_string(),
            body: body.to_string(),
        }
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.head.lines().skip(1).find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.trim()
                .eq_ignore_ascii_case(name)
                .then_some(value.trim())
        })
    }

    pub fn text(&self) -> String {
        format!("{}\r\n\r\n{}", self.head, self.body)
    }
}
