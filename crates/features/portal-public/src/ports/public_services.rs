use portal_model::Environment;

use crate::responses::PublicService;

pub trait PublicServices: Send + Sync {
    fn public_services(&self, environment: &Environment) -> Vec<PublicService>;
}
