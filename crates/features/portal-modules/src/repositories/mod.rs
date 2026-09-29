mod modules;

#[cfg(test)]
mod tests;

pub use modules::{LEGACY_KEY, SECTION, remove_legacy_switch, write_switch};
