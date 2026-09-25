use url::Url;

pub const TCP_DEFAULT_PORTS: [(&str, u16); 9] = [
    ("ssh", 22),
    ("telnet", 23),
    ("smb", 445),
    ("ldap", 389),
    ("mqtt", 1883),
    ("rdp", 3389),
    ("vnc", 5900),
    ("ipp", 631),
    ("postgres", 5432),
];

pub fn probe_target(url: &str, path: &str) -> Result<Url, String> {
    let base = Url::parse(url).map_err(|error| format!("invalid url: {error}"))?;
    let base_path = base.path().trim_end_matches('/');
    let extra = path.trim_start_matches('/');
    let mut target = base.clone();
    if extra.is_empty() {
        return Ok(target);
    }
    target.set_path(&format!("{base_path}/{extra}"));
    Ok(target)
}

pub fn probe_host(url: &str) -> Result<String, String> {
    let parsed = Url::parse(url).map_err(|error| format!("invalid url: {error}"))?;
    match parsed.host() {
        Some(url::Host::Ipv6(address)) => Ok(address.to_string()),
        Some(host) => Ok(host.to_string()),
        None => Err("the address names no host".to_string()),
    }
}

pub fn tcp_port(url: &str, port: Option<u16>) -> Result<u16, String> {
    if let Some(port) = port {
        return Ok(port);
    }
    let parsed = Url::parse(url).map_err(|error| format!("invalid url: {error}"))?;
    parsed
        .port_or_known_default()
        .or_else(|| {
            TCP_DEFAULT_PORTS
                .iter()
                .find(|(scheme, _)| *scheme == parsed.scheme())
                .map(|(_, port)| *port)
        })
        .ok_or_else(|| {
            format!(
                "no port: set probe.port, put a port in the address, or use a scheme with a known port ({})",
                parsed.scheme()
            )
        })
}
