use portal_feature::FieldError;
use portal_model::Publication;

pub trait Publishing: Send + Sync {
    fn https_port(&self) -> Option<u16>;

    fn problems(&self, publication: &Publication) -> Vec<FieldError>;
}
