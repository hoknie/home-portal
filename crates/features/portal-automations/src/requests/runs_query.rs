use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RunsQuery {
    #[serde(default)]
    pub automation: Option<String>,
    #[serde(default)]
    pub webhook: Option<String>,
    #[serde(default)]
    pub workflow: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub limit: Option<String>,
    #[serde(default)]
    pub before: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunsPage {
    pub limit: usize,
    pub before: Option<u64>,
}

impl RunsQuery {
    pub const DEFAULT_LIMIT: usize = 50;
    pub const LARGEST_LIMIT: usize = 200;

    pub fn page(&self) -> Result<RunsPage, (&'static str, &'static str)> {
        let limit = match &self.limit {
            None => Self::DEFAULT_LIMIT,
            Some(text) => text
                .parse::<usize>()
                .ok()
                .filter(|limit| (1..=Self::LARGEST_LIMIT).contains(limit))
                .ok_or(("limit", "must be a whole number from 1 to 200"))?,
        };
        let before = match &self.before {
            None => None,
            Some(text) => Some(
                text.parse::<u64>()
                    .map_err(|_| ("before", "must be a run id"))?,
            ),
        };
        Ok(RunsPage { limit, before })
    }
}
