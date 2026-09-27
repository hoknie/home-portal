mod modules;

#[cfg(test)]
mod tests;

pub use modules::{SECTION, remove_legacy_switch, write_switch};
