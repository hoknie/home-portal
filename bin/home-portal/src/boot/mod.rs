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
pub use listener::bind;
pub use router::{API_ANY_PATH, API_ROOT, assemble, rule_book};
pub use run::{prepare, run};
pub use serve::{Limits, accept_until};
pub use shutdown::requested;
pub use start::start;
