#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetValue {
    Text(String),
    Json(String),
    List(Vec<String>),
    Object(Vec<(String, String)>),
}
