mod atomic_write;
mod document;
mod entries;
mod layout;
mod location;
mod merge;
mod permissions;
mod placement;
mod positions;
mod secrets;
mod section;
mod storage;

#[cfg(test)]
mod tests;

pub use atomic_write::{create_private_folder, keep_previous, write_atomically};
pub use document::{parse_document, stamp_of};
pub use entries::{empty_entries, entry_id, unwrapped, wrapped};
pub use layout::{layout_errors, layout_of};
pub use location::{
    CONFIGURATION_VARIABLE, configuration_path, resolve_configuration_path, stray_configuration,
};
pub use merge::merge;
pub use permissions::refuse_if_readable;
pub use placement::misplaced;
pub use secrets::{holds_secrets, take_secrets};
pub use section::deserialize_section;
pub use storage::{storage_errors, storage_places};
