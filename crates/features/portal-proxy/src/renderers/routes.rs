use std::net::SocketAddr;

use serde_json::{Value, json};
use url::Url;

use crate::types::PublishedService;

pub const USER_HEADER: &str = "X-Portal-User";
pub const AUTHORIZE_PATH: &str = "/api/proxy/authorize";
pub const USER_PLACEHOLDER: &str = "{http.reverse_proxy.header.X-Portal-User}";

pub fn portal_dial(portal: SocketAddr) -> String {
    let address = match portal.ip() {
        ip if ip.is_unspecified() && ip.is_ipv4() => std::net::Ipv4Addr::LOCALHOST.into(),
        ip if ip.is_unspecified() => std::net::Ipv6Addr::LOCALHOST.into(),
        ip => ip,
    };
    SocketAddr::new(address, portal.port()).to_string()
}

pub fn portal_route(host: &str, portal: &str) -> Value {
    host_route(
        host,
        vec![
            delete_user(),
            json!({ "handler": "reverse_proxy", "upstreams": [{ "dial": portal }] }),
        ],
    )
}

pub fn service_route(service: &PublishedService, portal: &str) -> Value {
    let publication = &service.publication;
    let mut handlers = vec![delete_user()];
    if !publication.auth.is_empty() {
        handlers.push(forward_auth(portal));
    }
    handlers.push(upstream(&service.upstream, publication.upstream_verify));
    host_route(&publication.host, handlers)
}

pub fn forward_auth(portal: &str) -> Value {
    json!({
        "handle_response": [{
            "match": { "status_code": [2] },
            "routes": [
                { "handle": [{ "handler": "vars" }] },
                { "handle": [{ "handler": "headers", "request": { "delete": [USER_HEADER] } }] },
                {
                    "handle": [{
                        "handler": "headers",
                        "request": { "set": { USER_HEADER: [USER_PLACEHOLDER] } }
                    }],
                    "match": [{ "not": [{ "vars": { USER_PLACEHOLDER: [""] } }] }]
                }
            ]
        }],
        "handler": "reverse_proxy",
        "headers": {
            "request": {
                "set": {
                    "X-Forwarded-Method": ["{http.request.method}"],
                    "X-Forwarded-Uri": ["{http.request.uri}"]
                }
            }
        },
        "rewrite": { "method": "GET", "uri": AUTHORIZE_PATH },
        "upstreams": [{ "dial": portal }]
    })
}

fn upstream(address: &str, verify: bool) -> Value {
    let parsed = Url::parse(address).ok();
    let dial = parsed
        .as_ref()
        .and_then(|url| {
            Some(format!(
                "{}:{}",
                url.host_str()?,
                url.port_or_known_default()?
            ))
        })
        .unwrap_or_else(|| address.to_string());
    let mut handler = json!({ "handler": "reverse_proxy", "upstreams": [{ "dial": dial }] });
    if parsed.is_some_and(|url| url.scheme() == "https") {
        let tls = if verify {
            json!({})
        } else {
            json!({ "insecure_skip_verify": true })
        };
        handler["transport"] = json!({ "protocol": "http", "tls": tls });
    }
    handler
}

fn delete_user() -> Value {
    json!({ "handler": "headers", "request": { "delete": [USER_HEADER] } })
}

fn host_route(host: &str, handlers: Vec<Value>) -> Value {
    json!({
        "match": [{ "host": [host] }],
        "handle": [{ "handler": "subroute", "routes": [{ "handle": handlers }] }],
        "terminal": true
    })
}
