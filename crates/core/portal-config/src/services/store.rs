use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError, RwLock};

use portal_feature::{ApiError, Check, FieldError, Validator};
use toml_edit::DocumentMut;

use super::loading::{load, revision_of, writes_to};
use crate::helpers::{
    merge, stamp_of, storage_errors, storage_places, stray_configuration, take_secrets,
    write_atomically,
};
use crate::types::{
    ConfigError, ConfigurationLocation, Current, Revision, SecretString, Snapshot, Storage,
};

pub struct ConfigStore {
    main: PathBuf,
    validators: RwLock<Vec<Validator>>,
    checks: RwLock<Vec<Check>>,
    current: Mutex<Current>,
    secrets: RwLock<BTreeMap<String, SecretString>>,
    writing: tokio::sync::Mutex<()>,
    storage: BTreeMap<Storage, PathBuf>,
}

impl ConfigStore {
    pub const STALE_MESSAGE: &'static str =
        "the configuration changed since it was read; reload and try again";

    pub fn open_located(location: &ConfigurationLocation) -> Result<ConfigStore, ConfigError> {
        ConfigStore::open(&location.path).map_err(|error| match error {
            ConfigError::Missing { path, stray: None } if location.by_default => {
                ConfigError::Missing {
                    stray: env::current_dir()
                        .ok()
                        .and_then(|working| stray_configuration(&working)),
                    path,
                }
            }
            other => other,
        })
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<ConfigStore, ConfigError> {
        let main = path.into();
        let loaded = load(&main)?;
        let errors = storage_errors(&main, &loaded.snapshot.document);
        if !errors.is_empty() {
            return Err(ConfigError::Invalid { path: main, errors });
        }
        let storage = storage_places(&main, &loaded.snapshot.document);
        let stamps = loaded
            .sources
            .iter()
            .map(|source| stamp_of(&source.path))
            .collect();
        Ok(ConfigStore {
            main,
            validators: RwLock::new(Vec::new()),
            checks: RwLock::new(Vec::new()),
            current: Mutex::new(Current {
                sources: loaded.sources,
                snapshot: loaded.snapshot,
                stamps,
                problem: None,
            }),
            secrets: RwLock::new(loaded.secrets),
            writing: tokio::sync::Mutex::new(()),
            storage,
        })
    }

    pub fn adopt(&self, validators: Vec<Validator>) -> Result<(), ConfigError> {
        *self
            .validators
            .write()
            .unwrap_or_else(PoisonError::into_inner) = validators;
        self.revalidate()
    }

    pub fn adopt_checks(&self, checks: Vec<Check>) -> Result<(), ConfigError> {
        *self.checks.write().unwrap_or_else(PoisonError::into_inner) = checks;
        self.revalidate()
    }

    fn revalidate(&self) -> Result<(), ConfigError> {
        let errors = self.validate(&self.read().document);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::Invalid {
                path: self.main.clone(),
                errors,
            })
        }
    }

    pub fn path(&self) -> &Path {
        &self.main
    }

    pub fn storage(&self, kind: Storage) -> PathBuf {
        self.storage
            .get(&kind)
            .cloned()
            .unwrap_or_else(|| PathBuf::from(kind.default_name()))
    }

    pub fn paths(&self) -> Vec<PathBuf> {
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .sources
            .iter()
            .map(|source| source.path.clone())
            .collect()
    }

    pub fn writes_to(&self) -> PathBuf {
        writes_to(&self.main, &self.read().document)
    }

    pub fn secret(&self, name: &str) -> Option<SecretString> {
        self.read();
        self.secrets
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(name)
            .cloned()
    }

    pub fn secret_names(&self) -> Vec<String> {
        self.read();
        self.secrets
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .keys()
            .cloned()
            .collect()
    }

    pub fn read(&self) -> Snapshot {
        {
            let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
            let stamps: Vec<_> = current
                .sources
                .iter()
                .map(|source| stamp_of(&source.path))
                .collect();
            if stamps == current.stamps {
                return current.snapshot.clone();
            }
            current.stamps = stamps;
        }
        self.reload();
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .snapshot
            .clone()
    }

    pub fn problem(&self) -> Option<String> {
        self.read();
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .problem
            .clone()
    }

    pub async fn update<T>(
        &self,
        target: &Path,
        expected: &Revision,
        edit: impl FnOnce(&mut DocumentMut) -> Result<T, ApiError>,
    ) -> Result<(T, Snapshot), ApiError> {
        let _writing = self.writing.lock().await;
        let loaded = load(&self.main).map_err(|error| {
            ApiError::Conflict(format!("the configuration is invalid: {error}"))
        })?;
        if let Some(problem) = self.validate(&loaded.snapshot.document).first() {
            return Err(ApiError::Conflict(format!(
                "the configuration is invalid: {}: {}",
                problem.field, problem.message
            )));
        }
        if loaded.snapshot.revision != *expected {
            return Err(ApiError::Conflict(Self::STALE_MESSAGE.to_string()));
        }
        let mut sources = loaded.sources;
        let index = sources
            .iter()
            .position(|source| source.path == target)
            .ok_or_else(|| {
                ApiError::Internal(format!("{} is not a configuration file", target.display()))
            })?;
        let value = edit(&mut sources[index].document)?;
        let text = sources[index].document.to_string();
        let previous = std::mem::replace(&mut sources[index].bytes, text.clone().into_bytes());
        let (mut merged, origins) = merge(&sources).map_err(|error| {
            ApiError::Invalid(vec![FieldError::new("configuration", error.message())])
        })?;
        let secrets = take_secrets(&mut merged);
        let errors = self.validate(&merged);
        if !errors.is_empty() {
            return Err(ApiError::Invalid(errors));
        }
        write_atomically(target, &previous, &text).map_err(|error| {
            ApiError::Internal(format!("writing {}: {error}", target.display()))
        })?;
        let snapshot = Snapshot {
            document: Arc::new(merged),
            revision: revision_of(&sources),
            origins: Arc::new(origins),
        };
        let stamps = sources
            .iter()
            .map(|source| stamp_of(&source.path))
            .collect();
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        current.sources = sources;
        current.snapshot = snapshot.clone();
        current.stamps = stamps;
        current.problem = None;
        *self.secrets.write().unwrap_or_else(PoisonError::into_inner) = secrets;
        Ok((value, snapshot))
    }

    fn reload(&self) {
        let known = self
            .current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .snapshot
            .revision
            .clone();
        let loaded = match load(&self.main) {
            Ok(loaded) => loaded,
            Err(error) => {
                self.ignore(error.message());
                return;
            }
        };
        if loaded.snapshot.revision == known {
            self.current
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .problem = None;
            return;
        }
        let previous = std::mem::replace(
            &mut *self.secrets.write().unwrap_or_else(PoisonError::into_inner),
            loaded.secrets,
        );
        if let Some(error) = self.validate(&loaded.snapshot.document).first() {
            *self.secrets.write().unwrap_or_else(PoisonError::into_inner) = previous;
            self.ignore(format!("{}: {}", error.field, error.message));
            return;
        }
        tracing::info!(path = %self.main.display(), "configuration reloaded");
        let stamps = loaded
            .sources
            .iter()
            .map(|source| stamp_of(&source.path))
            .collect();
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        current.sources = loaded.sources;
        current.snapshot = loaded.snapshot;
        current.stamps = stamps;
        current.problem = None;
    }

    fn ignore(&self, problem: String) {
        tracing::warn!(path = %self.main.display(), %problem, "configuration edit ignored");
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .problem = Some(problem);
    }

    fn validate(&self, document: &DocumentMut) -> Vec<FieldError> {
        let mut errors = storage_errors(&self.main, document);
        errors.extend(
            self.validators
                .read()
                .unwrap_or_else(PoisonError::into_inner)
                .iter()
                .flat_map(|validator| validator(document)),
        );
        errors.extend(
            self.checks
                .read()
                .unwrap_or_else(PoisonError::into_inner)
                .iter()
                .flat_map(|check| check(document)),
        );
        errors
    }
}
