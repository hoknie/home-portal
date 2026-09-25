use std::sync::{Arc, PoisonError, RwLock};

use portal_config::{ConfigStore, Revision};
use serde_json::Value;
use time::OffsetDateTime;
use tokio::time::Instant;

use crate::clients::CaddyAdmin;
use crate::ports::PublishedServices;
use crate::renderers::{portal_dial, render};
use crate::responses::CaddyResponse;
use crate::services::{CaddyManager, read_settings};
use crate::types::{CaddyProblem, Platform, ProxySettings, ProxyView, SyncSetup, SyncState};

pub const HELD: &str = "something holds Caddy's admin address but does not answer; stop that process (a Caddy waiting for a password, for example) and the portal starts Caddy again";

pub struct CaddySync {
    configuration: Arc<ConfigStore>,
    services: Arc<dyn PublishedServices>,
    setup: SyncSetup,
    state: RwLock<SyncState>,
}

impl CaddySync {
    pub fn new(
        configuration: Arc<ConfigStore>,
        services: Arc<dyn PublishedServices>,
        setup: SyncSetup,
    ) -> CaddySync {
        CaddySync {
            configuration,
            services,
            setup,
            state: RwLock::new(SyncState::default()),
        }
    }

    pub fn settings(&self) -> ProxySettings {
        read_settings(&self.configuration.read().document).unwrap_or_default()
    }

    pub fn rendered(&self) -> Option<(ProxySettings, Value)> {
        let settings = self.settings();
        if !settings.enabled {
            return None;
        }
        let rendered = render(&settings, &self.services.published(), self.setup.portal);
        Some((settings, rendered))
    }

    pub fn state(&self) -> SyncState {
        self.state
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn view(&self) -> ProxyView {
        let settings = self.settings();
        let services = self.services.published();
        let rendered = settings
            .enabled
            .then(|| render(&settings, &services, self.setup.portal));
        let platform = Platform::current();
        let caddy = CaddyResponse {
            managed: settings.managed,
            installed: self.setup.manager.installed(),
            installed_from: self.setup.manager.installed_from(),
            source: settings.caddy.base.clone(),
            version: settings.caddy.version.to_string(),
            release_url: settings.caddy.release_url(),
            platform: platform.as_ref().ok().map(Platform::name),
            platform_error: platform.err(),
            download: self.setup.manager.download_state(),
            log: self.setup.manager.log(),
        };
        ProxyView {
            settings,
            services,
            rendered,
            state: self.state(),
            portal: portal_dial(self.setup.portal),
            caddy,
        }
    }

    pub fn known_hosts(&self) -> Vec<String> {
        let settings = self.settings();
        if !settings.enabled {
            return Vec::new();
        }
        settings
            .portal_host
            .into_iter()
            .chain(
                self.services
                    .published()
                    .into_iter()
                    .map(|service| service.publication.host),
            )
            .collect()
    }

    pub async fn apply(&self) -> Result<(), CaddyProblem> {
        let Some((settings, rendered)) = self.rendered() else {
            return Ok(());
        };
        let outcome = match CaddyAdmin::new(&settings.admin) {
            Ok(admin) => admin.load(&rendered).await,
            Err(message) => Err(CaddyProblem::Unreachable(message)),
        };
        self.record(outcome.clone().map(|()| rendered), true);
        outcome
    }

    pub async fn check(&self) -> Result<(), CaddyProblem> {
        let Some((settings, rendered)) = self.rendered() else {
            return Ok(());
        };
        let admin = CaddyAdmin::new(&settings.admin).map_err(CaddyProblem::Unreachable)?;
        match admin.config().await {
            Ok(current) if current == rendered => {
                self.record(Ok(rendered), false);
                Ok(())
            }
            Ok(_) => self.apply().await,
            Err(problem) => {
                self.record(Err(problem.clone()), false);
                Err(problem)
            }
        }
    }

    pub fn manager(&self) -> &Arc<CaddyManager> {
        &self.setup.manager
    }

    pub async fn revive(&self, settings: &ProxySettings) -> bool {
        if !settings.managed || self.setup.manager.installed().is_none() {
            return false;
        }
        if CaddyAdmin::occupied(&settings.admin).await {
            let Ok(admin) = CaddyAdmin::new(&settings.admin) else {
                return false;
            };
            if matches!(admin.config().await, Err(CaddyProblem::Unreachable(_))) {
                self.record(Err(CaddyProblem::Unreachable(HELD.to_string())), false);
            }
            return false;
        }
        match self.setup.manager.launch(&settings.admin) {
            Ok(()) => {
                tracing::info!("managed caddy started");
                true
            }
            Err(message) => {
                self.record(Err(CaddyProblem::Unreachable(message)), false);
                false
            }
        }
    }

    pub async fn run(self: Arc<Self>) {
        let cadence = self.setup.cadence;
        let mut seen: Option<Revision> = None;
        let mut pending = false;
        let mut checked = Instant::now();
        let mut attempted: Option<Instant> = None;
        let mut probed: Option<Instant> = None;
        loop {
            let revision = self.configuration.read().revision;
            let settings = self.settings();
            if settings.enabled {
                if seen.as_ref() != Some(&revision) {
                    seen = Some(revision);
                    pending = true;
                    attempted = None;
                }
                if probed.is_none_or(|at| at.elapsed() >= cadence.restart_every) {
                    probed = Some(Instant::now());
                    if self.revive(&settings).await {
                        pending = true;
                    }
                }
                if pending && attempted.is_none_or(|at| at.elapsed() >= cadence.retry_every) {
                    pending = self.apply().await.is_err();
                    attempted = Some(Instant::now());
                    checked = Instant::now();
                } else if !pending && checked.elapsed() >= cadence.check_every {
                    let _ = self.check().await;
                    checked = Instant::now();
                }
            } else {
                seen = None;
                pending = false;
            }
            tokio::time::sleep(cadence.tick).await;
        }
    }

    fn record(&self, outcome: Result<Value, CaddyProblem>, loaded: bool) {
        let mut state = self.state.write().unwrap_or_else(PoisonError::into_inner);
        match outcome {
            Ok(applied) => {
                if loaded {
                    state.last_applied_at = Some(OffsetDateTime::now_utc());
                }
                state.reachable = true;
                state.applied = Some(applied);
                state.last_error = None;
            }
            Err(problem) => {
                let message = problem.to_string();
                if state.last_error.as_ref() != Some(&message) {
                    tracing::warn!(%message, "caddy configuration not loaded");
                }
                state.reachable = problem.answered();
                state.last_error = Some(message);
            }
        }
    }
}
