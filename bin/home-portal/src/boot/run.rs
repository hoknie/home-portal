use super::{address, configuration, lifecycle, listener, logging, loops, router, serve};
use crate::features::registered;
use crate::types::{BootError, Ended, Signals, Wiring};

pub async fn run() -> Result<Ended, BootError> {
    let signals = Signals::install();
    logging::install();
    let store = configuration::open()?;
    let effective = address::from_environment(&store)?;
    let registry = registered(&Wiring {
        configuration: store.clone(),
        effective,
    })?;
    configuration::adopt(&store, &registry)?;
    let router = router::assemble(&registry);
    let listener = listener::bind(effective.address).await?;
    loops::spawn(&registry);
    lifecycle::started(&registry, effective.address);
    serve::serve(listener, router, &registry, signals).await
}
