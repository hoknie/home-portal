mod classify;
mod content_types;

#[cfg(test)]
mod tests;

pub use classify::looks_like_asset;
pub use content_types::{FALLBACK_CONTENT_TYPE, content_type_of};
