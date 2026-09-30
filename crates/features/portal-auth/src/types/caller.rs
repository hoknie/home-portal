use std::net::IpAddr;

use portal_feature::Principal;

#[derive(Clone, PartialEq, Eq)]
pub struct Caller {
    pub principal: Principal,
    pub token: Option<String>,
    pub address: Option<IpAddr>,
}
