mod address;
mod configuration;
mod interface;
mod lifecycle;
mod listener;
mod logging;
mod loops;
mod router;
mod run;
mod serve;
mod shutdown;
mod start;

#[cfg(test)]
mod tests;

pub use address::{ADDRESS_VARIABLE, parse_address, resolve_address};
pub use configuration::adopt;
pub use interface::located;
pub use router::assemble;
pub use run::run;
pub use start::start;
