use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tokio::sync::Notify;
use tokio::time::Instant;

use portal_config::ConfigStore;
use portal_model::Environment;

use super::StatusBoard;
use crate::loops::Prober;
use crate::probes::Probe;
use crate::types::{Running, ServiceEntry, ServicesSection, Wake};

pub struct Supervisor {
    configuration: Arc<ConfigStore>,
    host: Environment,
    board: Arc<StatusBoard>,
    probe: Arc<Probe>,
    running: Mutex<HashMap<String, Running>>,
    woken: Mutex<HashMap<String, Instant>>,
}

impl Supervisor {
    pub const TICK: Duration = Duration::from_secs(2);
    pub const WAKE_INTERVAL: Duration = Duration::from_secs(5);

    pub fn new(
        configuration: Arc<ConfigStore>,
        host: Environment,
        board: Arc<StatusBoard>,
        probe: Arc<Probe>,
    ) -> Supervisor {
        Supervisor {
            configuration,
            host,
            board,
            probe,
            running: Mutex::new(HashMap::new()),
            woken: Mutex::new(HashMap::new()),
        }
    }

    pub async fn run(self: Arc<Self>) {
        loop {
            self.reconcile();
            tokio::time::sleep(Self::TICK).await;
        }
    }

    pub fn reconcile(&self) {
        let wanted: Vec<ServiceEntry> = ServicesSection::read(&self.configuration.read().document)
            .map(|section| section.services)
            .unwrap_or_default()
            .into_iter()
            .filter(|entry| entry.probe.enabled)
            .collect();
        let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        running.retain(|id, task| {
            let keep = wanted.iter().any(|entry| entry.probes_like(&task.entry));
            if !keep {
                task.handle.abort();
                if !wanted.iter().any(|entry| &entry.id == id) {
                    self.board.forget(id);
                }
            }
            keep
        });
        for entry in wanted {
            if running.contains_key(&entry.id) {
                continue;
            }
            let wake = Arc::new(Notify::new());
            let handle = tokio::spawn(
                Prober {
                    entry: entry.clone(),
                    host: self.host.clone(),
                    probe: self.probe.clone(),
                    board: self.board.clone(),
                    wake: wake.clone(),
                }
                .run(),
            );
            running.insert(
                entry.id.clone(),
                Running {
                    entry,
                    handle,
                    wake,
                },
            );
        }
    }

    pub fn host(&self) -> &Environment {
        &self.host
    }

    pub fn wake(&self, id: &str) -> Wake {
        let configured = ServicesSection::read(&self.configuration.read().document)
            .map(|section| section.services)
            .unwrap_or_default()
            .into_iter()
            .find(|entry| entry.id == id);
        match configured {
            None => return Wake::NotFound,
            Some(entry) if !entry.probe.enabled => return Wake::Disabled,
            Some(_) => {}
        }
        let now = Instant::now();
        let mut woken = self.woken.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(last) = woken.get(id) {
            let since = now.duration_since(*last);
            if since < Self::WAKE_INTERVAL {
                return Wake::TooSoon(Self::WAKE_INTERVAL - since);
            }
        }
        self.reconcile();
        let running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(task) = running.get(id) else {
            return Wake::NotFound;
        };
        task.wake.notify_one();
        woken.insert(id.to_string(), now);
        Wake::Woken
    }

    #[cfg(test)]
    pub fn probing(&self) -> Vec<String> {
        let mut ids: Vec<String> = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .keys()
            .cloned()
            .collect();
        ids.sort();
        ids
    }
}
