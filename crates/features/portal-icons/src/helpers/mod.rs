mod digest;
mod inert;
mod sniff;

#[cfg(test)]
mod tests;

pub use digest::digest_of;
pub use inert::inert_headers;
pub use sniff::{SVG, extension_of, sniff, svg_is_inert};
