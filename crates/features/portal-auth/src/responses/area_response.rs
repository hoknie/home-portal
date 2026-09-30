use portal_feature::Area;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AreaResponse {
    pub area: &'static str,
    pub actions: Vec<&'static str>,
}

impl AreaResponse {
    pub fn matrix() -> Vec<AreaResponse> {
        Area::ALL
            .into_iter()
            .map(|area| AreaResponse {
                area: area.name(),
                actions: area.actions().iter().map(|action| action.name()).collect(),
            })
            .collect()
    }
}
