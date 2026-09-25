mod cache;
mod icons;
mod validation;

#[cfg(test)]
mod tests;

pub use cache::IconCache;
pub use icons::{CATALOG, Icons};
pub use validation::validate_icons;
