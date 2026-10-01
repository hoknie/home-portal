use std::collections::BTreeMap;
use std::env;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError, RwLock};

use portal_feature::{Check, FieldError, Validator};
use toml_edit::DocumentMut;

use super::loading::{load, read_main, workflow_files};
use crate::helpers::{
    layout_errors, layout_of, stamp_of, storage_errors, storage_places, stray_configuration,
};
use crate::types::{
    ConfigError, ConfigurationLocation, Current, Layout, Loaded, Origins, SecretString, Section,
    Snapshot, Source, Stamp, Storage,
};

pub struct ConfigStore {
    pub(super) main: PathBuf,
    pub(super) validators: RwLock<Vec<Validator>>,
    pub(super) checks: RwLock<Vec<Check>>,
    pub(super) current: Mutex<Current>,
    pub(super) reloading: Mutex<bool>,
    pub(super) secrets: RwLock<BTreeMap<String, SecretString>>,
    pub(super) writing: tokio::sync::Mutex<()>,
    pub(super) storage: BTreeMap<Storage, PathBuf>,
    pub(super) layout: Layout,
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
        let written = read_main(&main)?;
        let errors = layout_errors(&main, &written);
        if !errors.is_empty() {
            return Err(ConfigError::Invalid { path: main, errors });
        }
        let layout = layout_of(&main, &written);
        let loaded = load(&main, &layout)?;
        let errors = storage_errors(&main, &loaded.snapshot.document);
        if !errors.is_empty() {
            return Err(ConfigError::Invalid { path: main, errors });
        }
        let storage = storage_places(&main, &loaded.snapshot.document);
        let store = ConfigStore {
            validators: RwLock::new(Vec::new()),
            checks: RwLock::new(Vec::new()),
            current: Mutex::new(Current {
                sources: Vec::new(),
                snapshot: loaded.snapshot.clone(),
                stamps: Vec::new(),
                listing: Vec::new(),
                problem: None,
            }),
            reloading: Mutex::new(false),
            secrets: RwLock::new(BTreeMap::new()),
            writing: tokio::sync::Mutex::new(()),
            storage,
            layout,
            main,
        };
        store.adopt_loaded(loaded);
        Ok(store)
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
        let snapshot = self.read();
        let errors = self.validate(&snapshot.document);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::Invalid {
                path: self.main.clone(),
                errors: self.with_files(errors, &snapshot.origins),
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

    pub fn home_of(&self, section: Section) -> PathBuf {
        self.layout.file_of(section)
    }

    pub fn workflow_file(&self, id: &str) -> PathBuf {
        self.layout.workflow_file(id)
    }

    pub fn in_workflow_folder(&self, path: &Path) -> bool {
        self.layout.in_folder(path)
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

    pub const LOADER_FAILED: &'static str =
        "loading the changed configuration failed unexpectedly; the last good one is kept";

    pub fn read(&self) -> Snapshot {
        if let Some(snapshot) = self.unchanged() {
            return snapshot;
        }
        {
            let mut reloading = self
                .reloading
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if *reloading {
                return self.snapshot_now();
            }
            if let Some(snapshot) = self.unchanged() {
                return snapshot;
            }
            *reloading = true;
        }
        let _reloading = Reloading(&self.reloading);
        let seen = self.disk_state();
        let adopted = catch_unwind(AssertUnwindSafe(|| self.reload())).unwrap_or_else(|_| {
            self.ignore(Self::LOADER_FAILED.to_string());
            false
        });
        if !adopted {
            let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
            current.stamps = seen.0;
            current.listing = seen.1;
        }
        self.snapshot_now()
    }

    fn snapshot_now(&self) -> Snapshot {
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .snapshot
            .clone()
    }

    fn watched(&self, sources: &[Source]) -> Vec<PathBuf> {
        let mut paths = self.layout.files();
        for source in sources {
            if !paths.contains(&source.path) {
                paths.push(source.path.clone());
            }
        }
        paths
    }

    fn state_of(&self, sources: &[Source]) -> (Vec<Option<Stamp>>, Vec<PathBuf>) {
        let stamps = self
            .watched(sources)
            .iter()
            .map(|path| stamp_of(path))
            .collect();
        (stamps, workflow_files(&self.layout.folder()))
    }

    fn disk_state(&self) -> (Vec<Option<Stamp>>, Vec<PathBuf>) {
        let sources = self
            .current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .sources
            .clone();
        self.state_of(&sources)
    }

    fn unchanged(&self) -> Option<Snapshot> {
        let (stamps, listing) = self.disk_state();
        let current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        (stamps == current.stamps && listing == current.listing).then(|| current.snapshot.clone())
    }

    pub(super) fn adopt_loaded(&self, loaded: Loaded) {
        let (stamps, listing) = self.state_of(&loaded.sources);
        *self.secrets.write().unwrap_or_else(PoisonError::into_inner) = loaded.secrets;
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        current.sources = loaded.sources;
        current.snapshot = loaded.snapshot;
        current.stamps = stamps;
        current.listing = listing;
        current.problem = None;
    }

    pub fn problem(&self) -> Option<String> {
        self.read();
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .problem
            .clone()
    }

    pub(super) fn load_now(&self) -> Result<Loaded, ConfigError> {
        load(&self.main, &self.layout)
    }

    fn reload(&self) -> bool {
        let known = self.snapshot_now().revision;
        let loaded = match self.load_now() {
            Ok(loaded) => loaded,
            Err(error) => {
                self.ignore(error.message());
                return false;
            }
        };
        if loaded.snapshot.revision == known {
            self.current
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .problem = None;
            return false;
        }
        let previous = std::mem::replace(
            &mut *self.secrets.write().unwrap_or_else(PoisonError::into_inner),
            loaded.secrets.clone(),
        );
        let errors = self.validate(&loaded.snapshot.document);
        if let Some(error) = self.with_files(errors, &loaded.snapshot.origins).first() {
            *self.secrets.write().unwrap_or_else(PoisonError::into_inner) = previous;
            self.ignore(format!("{}: {}", error.field, error.message));
            return false;
        }
        tracing::info!(path = %self.main.display(), "configuration reloaded");
        self.adopt_loaded(loaded);
        true
    }

    fn ignore(&self, problem: String) {
        tracing::warn!(path = %self.main.display(), %problem, "configuration edit ignored");
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .problem = Some(problem);
    }

    pub(super) fn with_files(&self, errors: Vec<FieldError>, origins: &Origins) -> Vec<FieldError> {
        let prefix = format!("{}[", Section::Workflows.key());
        errors
            .into_iter()
            .map(|error| {
                let index = error
                    .field
                    .strip_prefix(&prefix)
                    .and_then(|rest| rest.split(']').next())
                    .and_then(|number| number.parse::<usize>().ok());
                match index.and_then(|index| origins.of(Section::Workflows.key(), index)) {
                    Some(file) if self.layout.in_folder(file) => FieldError::new(
                        error.field.clone(),
                        format!("{} (in {})", error.message, self.shown(file)),
                    ),
                    _ => error,
                }
            })
            .collect()
    }

    fn shown(&self, file: &Path) -> String {
        let base = self.main.parent().unwrap_or(Path::new(""));
        file.strip_prefix(base)
            .unwrap_or(file)
            .display()
            .to_string()
    }

    pub(super) fn validate(&self, document: &DocumentMut) -> Vec<FieldError> {
        let mut errors = storage_errors(&self.main, document);
        errors.extend(layout_errors(&self.main, document));
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

struct Reloading<'a>(&'a Mutex<bool>);

impl Drop for Reloading<'_> {
    fn drop(&mut self) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = false;
    }
}
