mod automation;
mod bounded;
mod folders;
mod full_disk;
mod local_network;
mod not_applicable;
mod volumes;

#[cfg(test)]
mod tests;

pub use automation::{AutomationCheck, OSASCRIPT};
pub use folders::FolderCheck;
pub use full_disk::{FullDiskCheck, PROTECTED_FILE};
pub use local_network::{LocalNetworkCheck, multicast};
pub use not_applicable::NotApplicableCheck;
pub use volumes::{VOLUMES, VolumesCheck, removable_by_diskutil, system_device};
