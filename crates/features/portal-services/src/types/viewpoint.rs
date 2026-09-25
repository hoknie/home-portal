use portal_model::Environment;

#[derive(Debug, Clone, Copy)]
pub struct Viewpoint<'a> {
    pub environment: &'a Environment,
    pub host: &'a Environment,
    pub publishing: Option<u16>,
}
