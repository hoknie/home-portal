use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::ProblemResponse;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FailureReportResponse {
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub since: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub checked: OffsetDateTime,
    pub details: bool,
    pub problems: Option<Vec<ProblemResponse>>,
}
