mod caddy_home;
mod caddy_source;
mod download_state;
mod platform;
mod release;
mod release_asset;

#[cfg(test)]
mod tests;

pub use caddy_home::CaddyHome;
pub use caddy_source::{CaddySource, CaddyVersion};
pub use download_state::{DownloadStage, DownloadState};
pub use platform::Platform;
pub use release::Release;
pub use release_asset::ReleaseAsset;
