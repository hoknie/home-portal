mod answer;
mod certificate;
mod handler;
mod library;
mod settings;
mod zone_book;

#[cfg(test)]
pub mod tests;

pub use answer::answer;
pub use certificate::{locate, server_config};
pub use handler::{UDP_LIMIT, handle, respond};
pub use library::Library;
pub use settings::{read_settings, validate_dns};
pub use zone_book::build;
