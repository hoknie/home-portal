use portal_model::Publication;
use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RawPublishedService {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub proxy: Option<Publication>,
}
