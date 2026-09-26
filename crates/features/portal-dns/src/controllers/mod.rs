mod dns;
mod dns_query;

#[cfg(test)]
mod tests;

pub use dns::{change, show};
pub use dns_query::{resolve_encoded, resolve_posted};
