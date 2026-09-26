use std::env;
use std::process::ExitCode;
use std::sync::Arc;

use portal_config::{ConfigStore, configuration_path};
use portal_network::{host_environment, read_environments};
use portal_proxy::{read_settings, render};
use portal_services::ServicesSection;

use crate::adapters::ServicePublications;
use crate::boot::{ADDRESS_VARIABLE, adopt, resolve_address};
use crate::features::registered;
use crate::types::Wiring;

pub const PROXY: &str = "proxy";
pub const RENDER: &str = "render";
pub const USAGE: &str = "usage: home-portal proxy render";
pub const DISABLED: &str = "the proxy is not enabled; set enabled = true in the [proxy] section to render its configuration";

pub fn proxy(arguments: &[String]) -> ExitCode {
    if arguments != [RENDER] {
        eprintln!("home-portal: {USAGE}");
        return ExitCode::FAILURE;
    }
    match rendered() {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("home-portal: {message}");
            ExitCode::FAILURE
        }
    }
}

fn rendered() -> Result<String, String> {
    let store = Arc::new(
        configuration_path()
            .and_then(|location| ConfigStore::open_located(&location))
            .map_err(|error| error.to_string())?,
    );
    let effective = resolve_address(&store, env::var(ADDRESS_VARIABLE).ok())
        .map_err(|error| error.to_string())?;
    let registry = registered(&Wiring {
        configuration: store.clone(),
        effective,
    })
    .map_err(|error| error.to_string())?;
    adopt(&store, &registry).map_err(|error| error.to_string())?;
    let document = store.read().document;
    let settings = read_settings(&document).map_err(|errors| format!("{errors:?}"))?;
    if !settings.enabled {
        return Err(DISABLED.to_string());
    }
    let entries = ServicesSection::read(&document)?.services;
    let host = host_environment(&read_environments(&document).unwrap_or_default());
    let configuration = render(
        &settings,
        &ServicePublications::of(entries, &host),
        effective.address,
    );
    serde_json::to_string_pretty(&configuration).map_err(|error| error.to_string())
}
