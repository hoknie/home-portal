use portal_feature::Module;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleView {
    pub module: Module,
    pub enabled: bool,
    pub requires: Vec<Module>,
    pub required_by: Vec<Module>,
}
