use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, PoisonError, RwLock};
use std::time::Duration;

use super::caddy_installer::install;
use super::caddy_launcher::{launch, log_tail};
use crate::clients::Releases;
use crate::types::{AdminAddress, CaddyHome, CaddySource, DownloadStage, DownloadState, Platform};

pub struct CaddyManager {
    home: CaddyHome,
    download: RwLock<DownloadState>,
    busy: AtomicBool,
}

impl CaddyManager {
    pub const DOWNLOAD_CEILING: Duration = Duration::from_secs(600);
    pub const LOG_LINES: usize = 20;
    pub const BUSY: &'static str = "Caddy is already being downloaded";

    pub fn new(home: CaddyHome) -> CaddyManager {
        CaddyManager {
            home,
            download: RwLock::new(DownloadState::default()),
            busy: AtomicBool::new(false),
        }
    }

    pub fn home(&self) -> &CaddyHome {
        &self.home
    }

    pub fn installed(&self) -> Option<String> {
        self.home.installed()
    }

    pub fn installed_from(&self) -> Option<String> {
        self.home.installed_from()
    }

    pub fn download_state(&self) -> DownloadState {
        self.download
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn log(&self) -> Vec<String> {
        log_tail(&self.home, Self::LOG_LINES)
    }

    pub fn launch(&self, admin: &AdminAddress) -> Result<(), String> {
        launch(&self.home, admin).map_err(|error| format!("cannot start Caddy: {error}"))
    }

    pub fn begin_download(self: &Arc<Self>, source: CaddySource) -> Result<(), &'static str> {
        if self.busy.swap(true, Ordering::SeqCst) {
            return Err(Self::BUSY);
        }
        self.set(DownloadStage::Downloading, None);
        let manager = self.clone();
        tokio::spawn(async move {
            let outcome = match Platform::current() {
                Ok(platform) => manager.run_download(&platform, &source).await,
                Err(message) => Err(message),
            };
            match outcome {
                Ok(version) => {
                    tracing::info!(%version, "caddy installed");
                    manager.set(DownloadStage::Installed, None);
                }
                Err(message) => {
                    tracing::warn!(%message, "caddy download failed");
                    manager.set(DownloadStage::Failed, Some(message));
                }
            }
            manager.busy.store(false, Ordering::SeqCst);
        });
        Ok(())
    }

    pub async fn run_download(
        &self,
        platform: &Platform,
        source: &CaddySource,
    ) -> Result<String, String> {
        let releases = Releases::new()?;
        tokio::time::timeout(
            Self::DOWNLOAD_CEILING,
            install(&self.home, &releases, platform, source),
        )
        .await
        .map_err(|_| {
            format!(
                "the download took longer than {} minutes",
                Self::DOWNLOAD_CEILING.as_secs() / 60
            )
        })?
    }

    fn set(&self, state: DownloadStage, error: Option<String>) {
        *self
            .download
            .write()
            .unwrap_or_else(PoisonError::into_inner) = DownloadState { state, error };
    }
}
