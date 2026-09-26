use portal_model::Environment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedHost {
    pub host: String,
    pub environments: Option<Vec<Environment>>,
}
