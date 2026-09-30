use portal_feature::Principal;

#[derive(Clone, PartialEq, Eq)]
pub struct Caller {
    pub principal: Principal,
    pub token: Option<String>,
}
