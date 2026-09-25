use std::collections::BTreeMap;
use std::net::SocketAddr;

use portal_model::{TlsMode, TlsPolicy};
use serde_json::{Value, json};

use super::routes::{portal_dial, portal_route, service_route};
use crate::types::{ProxySettings, PublishedService};

pub const SERVER: &str = "home-portal";
pub const LOCAL_AUTHORITY: &str = "local";

pub fn render(
    settings: &ProxySettings,
    services: &[PublishedService],
    portal: SocketAddr,
) -> Value {
    let portal = portal_dial(portal);
    let mut routes = Vec::new();
    let mut policies: BTreeMap<TlsPolicy, Vec<String>> = BTreeMap::new();
    if let Some(host) = &settings.portal_host {
        routes.push(portal_route(host, &portal));
        policies
            .entry(settings.tls.clone())
            .or_default()
            .push(host.clone());
    }
    for service in services {
        routes.push(service_route(service, &portal));
        policies
            .entry(settings.tls_of(&service.publication))
            .or_default()
            .push(service.publication.host.clone());
    }
    let listen = format!(":{}", settings.https_port);
    let mut server = json!({ "listen": [listen], "routes": routes });
    let certificates = files_of(&policies);
    if !certificates.is_empty() {
        let mut selections: Vec<Value> = certificates
            .iter()
            .map(|(tag, _, hosts)| {
                json!({ "match": { "sni": hosts }, "certificate_selection": { "any_tag": [tag] } })
            })
            .collect();
        selections.push(json!({}));
        server["tls_connection_policies"] = Value::Array(selections);
    }
    let mut apps = json!({
        "http": {
            "http_port": settings.http_port,
            "https_port": settings.https_port,
            "servers": { SERVER: server }
        },
        "tls": tls_app(&policies, &certificates)
    });
    if policies
        .keys()
        .any(|policy| policy.mode == TlsMode::Internal)
    {
        apps["pki"] =
            json!({ "certificate_authorities": { LOCAL_AUTHORITY: { "install_trust": false } } });
    }
    json!({
        "admin": { "listen": settings.admin.listen() },
        "apps": apps
    })
}

fn files_of(policies: &BTreeMap<TlsPolicy, Vec<String>>) -> Vec<(String, &TlsPolicy, Vec<String>)> {
    policies
        .iter()
        .filter(|(policy, _)| policy.mode == TlsMode::Files)
        .enumerate()
        .map(|(index, (policy, hosts))| (format!("{SERVER}-{index}"), policy, sorted(hosts)))
        .collect()
}

fn tls_app(
    policies: &BTreeMap<TlsPolicy, Vec<String>>,
    certificates: &[(String, &TlsPolicy, Vec<String>)],
) -> Value {
    let automation: Vec<Value> = policies
        .iter()
        .filter_map(|(policy, hosts)| {
            let issuer = match policy.mode {
                TlsMode::Acme => match &policy.email {
                    Some(email) => json!({ "module": "acme", "email": email }),
                    None => json!({ "module": "acme" }),
                },
                TlsMode::Internal => json!({ "module": "internal" }),
                TlsMode::Files => return None,
            };
            Some(json!({ "subjects": sorted(hosts), "issuers": [issuer] }))
        })
        .collect();
    let mut app = json!({ "automation": { "policies": automation } });
    if !certificates.is_empty() {
        let files: Vec<Value> = certificates
            .iter()
            .map(|(tag, policy, _)| {
                json!({
                    "certificate": policy.certificate.as_deref().unwrap_or_default(),
                    "key": policy.key.as_deref().unwrap_or_default(),
                    "tags": [tag]
                })
            })
            .collect();
        app["certificates"] = json!({ "load_files": files });
    }
    app
}

fn sorted(hosts: &[String]) -> Vec<String> {
    let mut hosts = hosts.to_vec();
    hosts.sort();
    hosts.dedup();
    hosts
}
