use ipnet::IpNet;
use portal_config::deserialize_section;
use portal_feature::FieldError;
use portal_model::{Environment, Environments, RawEnvironmentsSection};
use toml_edit::DocumentMut;

pub const SECTION: &str = "environments";

pub fn validate_environments(document: &DocumentMut) -> Vec<FieldError> {
    match read_environments(document) {
        Ok(_) => Vec::new(),
        Err(errors) => errors,
    }
}

pub fn read_environments(document: &DocumentMut) -> Result<Environments, Vec<FieldError>> {
    let section: RawEnvironmentsSection =
        deserialize_section(document).map_err(|message| vec![FieldError::new(SECTION, message)])?;
    let mut errors = Vec::new();
    let mut named: Vec<(Environment, Vec<IpNet>)> = Vec::new();
    for (name, raw) in &section.environments {
        let field = format!("{SECTION}.{name}");
        if name == Environment::INTERNET {
            errors.push(FieldError::new(&field, "is the name of every visitor matched by no other environment, and cannot be configured"));
            continue;
        }
        let environment = match Environment::parse(name) {
            Ok(environment) => environment,
            Err(problem) => {
                errors.push(FieldError::new(&field, problem.to_string()));
                continue;
            }
        };
        let mut networks = Vec::new();
        for (index, text) in raw.networks.iter().enumerate() {
            match parse_network(text) {
                Some(network) => networks.push(network),
                None => errors.push(FieldError::new(
                    format!("{field}.networks[{index}]"),
                    "must be an IP address or a CIDR range such as 192.168.0.0/16",
                )),
            }
        }
        if networks.is_empty() && errors.is_empty() {
            errors.push(FieldError::new(
                format!("{field}.networks"),
                "must list at least one address or range",
            ));
        }
        named.push((environment, networks));
    }
    errors.extend(overlaps(&named));
    if errors.is_empty() {
        Ok(Environments::new(named))
    } else {
        Err(errors)
    }
}

fn overlaps(named: &[(Environment, Vec<IpNet>)]) -> Vec<FieldError> {
    let mut errors = Vec::new();
    for (index, (environment, networks)) in named.iter().enumerate() {
        for (other, other_networks) in named.iter().skip(index + 1) {
            for network in networks {
                for candidate in other_networks {
                    if network.contains(&candidate.network())
                        || candidate.contains(&network.network())
                    {
                        errors.push(FieldError::new(
                            format!("{SECTION}.{environment}.networks"),
                            format!("{network} overlaps {candidate} of {other}; an address belongs to one environment"),
                        ));
                    }
                }
            }
        }
    }
    errors
}

fn parse_network(text: &str) -> Option<IpNet> {
    let text = text.trim();
    text.parse::<IpNet>()
        .ok()
        .or_else(|| text.parse::<std::net::IpAddr>().ok().map(IpNet::from))
}
