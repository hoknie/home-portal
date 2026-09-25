use super::{address, configuration, listener, logging, loops, router, serve};
use crate::features::registered;
use crate::types::{BootError, Wiring};

pub async fn run() -> Result<(), BootError> {
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
    serve::serve(listener, router, &registry).await
}
