use std::net::SocketAddr;

use super::{address, configuration, lifecycle, listener, logging, loops, router, serve};
use crate::failures;
use crate::features::registered;
use crate::types::{BootError, Ended, Registry, Signals, Wiring};

pub struct Prepared {
    pub registry: Registry,
    pub address: SocketAddr,
}

pub async fn run() -> Result<Ended, BootError> {
    let signals = Signals::install();
    logging::install();
    match prepare() {
        Ok(prepared) => started(prepared, signals).await,
        Err(error) if error.enters_failure_mode() => failures::serve(error, signals).await,
        Err(error) => Err(error),
    }
}

pub fn prepare() -> Result<Prepared, BootError> {
    let store = configuration::open()?;
    let effective = address::from_environment(&store)?;
    let registry = registered(&Wiring {
        configuration: store.clone(),
        effective,
    })?;
    configuration::adopt(&store, &registry)?;
    Ok(Prepared {
        registry,
        address: effective.address,
    })
}

async fn started(prepared: Prepared, signals: Signals) -> Result<Ended, BootError> {
    let Prepared { registry, address } = prepared;
    let router = router::assemble(&registry);
    let listener = listener::bind(address).await?;
    loops::spawn(&registry);
    lifecycle::started(&registry, address);
    serve::serve(listener, router, &registry, signals).await
}
