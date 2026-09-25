use portal_model::Environments;

#[derive(Debug, Clone, Default)]
pub struct Known {
    pub environments: Environments,
    pub widgets: Vec<String>,
}
