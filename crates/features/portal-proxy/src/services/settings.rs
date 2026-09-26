use portal_config::deserialize_section;
use portal_feature::FieldError;
use portal_model::Publication;
use toml_edit::DocumentMut;

use crate::helpers::covers_loopback;
use crate::types::{
    AdminAddress, CaddySource, CaddyVersion, ProxySettings, RawDnsView, RawNetworkView, RawProxy,
    RawProxySection,
};

pub const SECTION: &str = "proxy";
pub const SECTION_PREFIX: &str = "proxy.";
pub const REQUIRED: &str = "is required when the proxy is enabled";
pub const NOT_A_PARENT: &str = "must be portal_host or a parent domain of it";
pub const LOOPBACK_NOT_TRUSTED: &str =
    "must cover 127.0.0.1 when the proxy is enabled, because Caddy forwards from there";
pub const PUBLIC_URL_DIFFERS: &str = "must be https://<proxy.portal_host>, with proxy.https_port when it is not 443, when the proxy is enabled";
pub const PORT_RANGE: &str = "must be between 1 and 65535";
pub const SAME_PORTS: &str = "must differ from proxy.http_port";

pub fn read_settings(document: &DocumentMut) -> Result<ProxySettings, Vec<FieldError>> {
    let section: RawProxySection =
        deserialize_section(document).map_err(|message| vec![FieldError::new(SECTION, message)])?;
    let mut errors = Vec::new();
    let mut settings = check_proxy(&section.proxy, &mut errors);
    settings.doh_host = doh_host(&section.dns, settings.portal_host.as_deref());
    if settings.enabled {
        errors.extend(check_network(&section.network, &settings));
    }
    if errors.is_empty() {
        Ok(settings)
    } else {
        Err(errors)
    }
}

pub fn validate_settings(document: &DocumentMut) -> Vec<FieldError> {
    read_settings(document).err().unwrap_or_default()
}

fn check_proxy(raw: &RawProxy, errors: &mut Vec<FieldError>) -> ProxySettings {
    let mut settings = ProxySettings {
        enabled: raw.enabled.unwrap_or(false),
        managed: raw.managed.unwrap_or(false),
        tls: raw.tls.clone().unwrap_or_default(),
        ..ProxySettings::default()
    };
    let field = |name: &str| format!("{SECTION_PREFIX}{name}");
    for (name, raw_port, target) in [
        ("http_port", raw.http_port, &mut settings.http_port),
        ("https_port", raw.https_port, &mut settings.https_port),
    ] {
        if let Some(port) = raw_port {
            match u16::try_from(port).ok().filter(|port| *port != 0) {
                Some(port) => *target = port,
                None => errors.push(FieldError::new(field(name), PORT_RANGE)),
            }
        }
    }
    if settings.http_port == settings.https_port {
        errors.push(FieldError::new(field("https_port"), SAME_PORTS));
    }
    if let Some(admin) = &raw.admin {
        match AdminAddress::parse(admin) {
            Ok(admin) => settings.admin = admin,
            Err(message) => errors.push(FieldError::new(field("admin"), message)),
        }
    }
    settings.portal_host = host_of(raw.portal_host.as_deref(), &field("portal_host"), errors);
    if settings.enabled && settings.portal_host.is_none() && raw.portal_host.is_none() {
        errors.push(FieldError::new(field("portal_host"), REQUIRED));
    }
    settings.cookie_domain = host_of(
        raw.cookie_domain.as_deref(),
        &field("cookie_domain"),
        errors,
    );
    if let (Some(domain), Some(host)) = (&settings.cookie_domain, &settings.portal_host)
        && !Publication::is_within(host, domain)
    {
        errors.push(FieldError::new(field("cookie_domain"), NOT_A_PARENT));
    }
    for (name, message) in settings.tls.problems() {
        errors.push(FieldError::new(field(&format!("tls.{name}")), message));
    }
    settings.caddy = caddy_source(raw, &field, errors);
    settings
}

fn caddy_source(
    raw: &RawProxy,
    field: &dyn Fn(&str) -> String,
    errors: &mut Vec<FieldError>,
) -> CaddySource {
    let mut source = CaddySource::default();
    let Some(caddy) = &raw.caddy else {
        return source;
    };
    if let Some(base) = &caddy.source {
        match CaddySource::parse_base(base) {
            Ok(base) => source.base = base,
            Err(message) => errors.push(FieldError::new(field("caddy.source"), message)),
        }
    }
    if let Some(version) = &caddy.version {
        match CaddyVersion::parse(version) {
            Ok(version) => source.version = version,
            Err(message) => errors.push(FieldError::new(field("caddy.version"), message)),
        }
    }
    source
}

fn host_of(text: Option<&str>, field: &str, errors: &mut Vec<FieldError>) -> Option<String> {
    let host = text?.trim();
    match Publication::host_problem(host) {
        None => Some(host.to_string()),
        Some(message) => {
            errors.push(FieldError::new(field, message));
            None
        }
    }
}

fn check_network(network: &RawNetworkView, settings: &ProxySettings) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let trusted = covers_loopback(network.trusted_proxies.iter().flatten().map(String::as_str));
    if !trusted {
        errors.push(FieldError::new(
            "network.trusted_proxies",
            LOOPBACK_NOT_TRUSTED,
        ));
    }
    if let (Some(url), Some(host)) = (
        network
            .public_url
            .as_deref()
            .map(str::trim)
            .filter(|url| !url.is_empty()),
        &settings.portal_host,
    ) && url.trim_end_matches('/') != settings.origin(host)
    {
        errors.push(FieldError::new("network.public_url", PUBLIC_URL_DIFFERS));
    }
    errors
}

fn doh_host(dns: &RawDnsView, portal_host: Option<&str>) -> Option<String> {
    let serving = dns.enabled.unwrap_or(false) && dns.https.enabled.unwrap_or(false);
    let host = dns
        .https
        .host
        .as_deref()
        .map(|host| host.trim().trim_end_matches('.').to_ascii_lowercase())
        .filter(|host| !host.is_empty())?;
    (serving && portal_host.is_none_or(|portal| !portal.eq_ignore_ascii_case(&host)))
        .then_some(host)
}
