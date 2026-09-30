use std::sync::Arc;

use portal_feature::ApiError;
use tokio::sync::{Mutex, OwnedMutexGuard};
use tokio::task::JoinSet;

use super::ReadPermissionSettings;
use crate::ports::CheckSource;
use crate::services::{PermissionBoard, asked};
use crate::types::{Finding, Limits, Owner};

pub const BUSY: &str = "permissions are being asked for already; wait for the answers";

#[derive(Clone)]
pub struct RequestPermissions {
    settings: ReadPermissionSettings,
    board: PermissionBoard,
    source: Arc<dyn CheckSource>,
    owner: Owner,
    limits: Limits,
    running: Arc<Mutex<()>>,
}

impl RequestPermissions {
    pub fn new(
        settings: ReadPermissionSettings,
        board: PermissionBoard,
        source: Arc<dyn CheckSource>,
        owner: Owner,
        limits: Limits,
    ) -> RequestPermissions {
        RequestPermissions {
            settings,
            board,
            source,
            owner,
            limits,
            running: Arc::new(Mutex::new(())),
        }
    }

    pub async fn run(&self) -> Result<(), ApiError> {
        let guard = self.claim()?;
        self.asked(true, guard).await;
        Ok(())
    }

    pub fn start(&self) -> Result<(), ApiError> {
        let guard = self.claim()?;
        for check in self.source.checks(&self.settings.run()) {
            let code = check.code();
            if code.requestable() && !self.board.asking(&code) {
                self.board.record(code, Finding::pending());
            }
        }
        let this = self.clone();
        tokio::spawn(async move { this.asked(true, guard).await });
        Ok(())
    }

    pub async fn at_start(&self) {
        let requesting = self.settings.run().request_at_start;
        if let Ok(guard) = self.claim() {
            self.asked(requesting, guard).await;
        }
    }

    fn claim(&self) -> Result<OwnedMutexGuard<()>, ApiError> {
        self.running
            .clone()
            .try_lock_owned()
            .map_err(|_| ApiError::Conflict(BUSY.to_string()))
    }

    async fn asked(&self, requesting: bool, guard: OwnedMutexGuard<()>) {
        let mut asking = JoinSet::new();
        for check in self.source.checks(&self.settings.run()) {
            if !requesting && check.code().requestable() {
                continue;
            }
            asking.spawn(asked(
                check,
                self.board.clone(),
                self.limits.prompt,
                self.owner.clone(),
            ));
        }
        while asking.join_next().await.is_some() {}
        drop(guard);
    }
}
