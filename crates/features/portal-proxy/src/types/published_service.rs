use portal_model::Publication;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedService {
    pub id: String,
    pub upstream: String,
    pub publication: Publication,
}
