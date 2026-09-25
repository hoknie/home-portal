mod digest;
mod sniff;

#[cfg(test)]
mod tests;

pub use digest::digest_of;
pub use sniff::{extension_of, sniff};
