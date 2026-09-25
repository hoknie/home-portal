use portal_model::{Environment, Environments};

pub fn host_environment(environments: &Environments) -> Environment {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .map(|interface| environments.of(interface.ip()))
        .find(|environment| !environment.is_internet())
        .unwrap_or_else(Environment::internet)
}
