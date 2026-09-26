use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NameResponse {
    pub name: String,
    pub answers: Vec<EnvironmentAnswerResponse>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentAnswerResponse {
    pub environment: String,
    pub records: Vec<RecordResponse>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
}
