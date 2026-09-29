use std::fs;
use std::path::Path;

use portal_feature::{ApiError, FieldError};
use toml_edit::DocumentMut;

use super::store::ConfigStore;
use crate::helpers::{
    create_private_folder, empty_entries, keep_previous, merge, take_secrets, write_atomically,
};
use crate::types::{Loaded, Revision, Shape, Snapshot, Source};

impl ConfigStore {
    pub const NO_SUCH_FILE: &'static str = "no such configuration file";

    fn current_for_write(&self, expected: &Revision) -> Result<Loaded, ApiError> {
        let loaded = self.load_now().map_err(|error| {
            ApiError::Conflict(format!("the configuration is invalid: {error}"))
        })?;
        let errors = self.with_files(
            self.validate(&loaded.snapshot.document),
            &loaded.snapshot.origins,
        );
        if let Some(problem) = errors.first() {
            return Err(ApiError::Conflict(format!(
                "the configuration is invalid: {}: {}",
                problem.field, problem.message
            )));
        }
        if loaded.snapshot.revision != *expected {
            return Err(ApiError::Conflict(Self::STALE_MESSAGE.to_string()));
        }
        Ok(loaded)
    }

    fn new_source(&self, target: &Path) -> Result<Source, ApiError> {
        let (document, shape) = if self.layout.files().iter().any(|file| file == target) {
            (DocumentMut::new(), Shape::Whole)
        } else if self.layout.in_folder(target) {
            (empty_entries(), Shape::Entry)
        } else {
            return Err(ApiError::Internal(format!(
                "{} is not a configuration file",
                target.display()
            )));
        };
        Ok(Source {
            path: target.to_path_buf(),
            document,
            bytes: Vec::new(),
            shape,
        })
    }

    fn checked(&self, sources: &[Source]) -> Result<(), ApiError> {
        let (mut merged, _) = merge(sources).map_err(|error| {
            ApiError::Invalid(vec![FieldError::new("configuration", error.message())])
        })?;
        take_secrets(&mut merged);
        let errors = self.validate(&merged);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ApiError::Invalid(errors))
        }
    }

    fn written(&self, path: &Path, source: &Source) -> Result<(), ApiError> {
        if let Some(folder) = path
            .parent()
            .filter(|folder| !folder.as_os_str().is_empty())
        {
            create_private_folder(folder).map_err(|error| {
                ApiError::Internal(format!("creating {}: {error}", folder.display()))
            })?;
        }
        write_atomically(path, &source.bytes, &source.text())
            .map_err(|error| ApiError::Internal(format!("writing {}: {error}", path.display())))
    }

    fn settled(&self) -> Result<Snapshot, ApiError> {
        let loaded = self.load_now().map_err(|error| {
            ApiError::Internal(format!("reading back the configuration: {error}"))
        })?;
        let snapshot = loaded.snapshot.clone();
        self.adopt_loaded(loaded);
        Ok(snapshot)
    }

    pub async fn update<T>(
        &self,
        target: &Path,
        expected: &Revision,
        edit: impl FnOnce(&mut DocumentMut) -> Result<T, ApiError>,
    ) -> Result<(T, Snapshot), ApiError> {
        let _writing = self.writing.lock().await;
        let mut sources = self.current_for_write(expected)?.sources;
        let index = match sources.iter().position(|source| source.path == target) {
            Some(index) => index,
            None => {
                sources.push(self.new_source(target)?);
                sources.len() - 1
            }
        };
        let value = edit(&mut sources[index].document)?;
        self.checked(&sources)?;
        self.written(target, &sources[index])?;
        Ok((value, self.settled()?))
    }

    pub async fn remove(&self, target: &Path, expected: &Revision) -> Result<Snapshot, ApiError> {
        let _writing = self.writing.lock().await;
        let mut sources = self.current_for_write(expected)?.sources;
        let index = sources
            .iter()
            .position(|source| source.path == target && source.shape == Shape::Entry)
            .ok_or(ApiError::NotFound(Self::NO_SUCH_FILE))?;
        let removed = sources.remove(index);
        self.checked(&sources)?;
        keep_previous(target, &removed.bytes)
            .and_then(|()| fs::remove_file(target))
            .map_err(|error| {
                ApiError::Internal(format!("removing {}: {error}", target.display()))
            })?;
        self.settled()
    }

    pub async fn update_moved<T>(
        &self,
        from: &Path,
        to: &Path,
        expected: &Revision,
        edit: impl FnOnce(&mut DocumentMut) -> Result<T, ApiError>,
    ) -> Result<(T, Snapshot), ApiError> {
        let _writing = self.writing.lock().await;
        let mut sources = self.current_for_write(expected)?.sources;
        if to.exists() || sources.iter().any(|source| source.path == to) {
            return Err(ApiError::Conflict(format!(
                "{} already exists",
                to.display()
            )));
        }
        let index = sources
            .iter()
            .position(|source| source.path == from && source.shape == Shape::Entry)
            .ok_or(ApiError::NotFound(Self::NO_SUCH_FILE))?;
        let old = sources[index].bytes.clone();
        let value = edit(&mut sources[index].document)?;
        sources[index].path = to.to_path_buf();
        sources[index].bytes = Vec::new();
        self.checked(&sources)?;
        self.written(to, &sources[index])?;
        if let Err(error) = keep_previous(from, &old).and_then(|()| fs::remove_file(from)) {
            let _ = fs::remove_file(to);
            return Err(ApiError::Internal(format!(
                "removing {}: {error}",
                from.display()
            )));
        }
        Ok((value, self.settled()?))
    }
}
