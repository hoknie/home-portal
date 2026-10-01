use std::path::{Path, PathBuf};
use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned, Snapshot};
use portal_feature::{ApiError, Module, ModulePreparer, ModuleSwitches};

use super::current_modules::views;
use crate::repositories::{SECTION, write_switch};
use crate::types::ModuleView;

#[derive(Clone)]
pub struct SwitchModule {
    configuration: Arc<ConfigStore>,
    preparers: Arc<Vec<Arc<dyn ModulePreparer>>>,
}

impl SwitchModule {
    pub fn new(
        configuration: Arc<ConfigStore>,
        preparers: Vec<Arc<dyn ModulePreparer>>,
    ) -> SwitchModule {
        SwitchModule {
            configuration,
            preparers: Arc::new(preparers),
        }
    }

    pub async fn run(
        &self,
        module: Module,
        on: bool,
        revision: &Revision,
    ) -> Result<Revisioned<Vec<ModuleView>>, ApiError> {
        let snapshot = self.configuration.read();
        let switches = ModuleSwitches::resolve(&snapshot.document).map_err(ApiError::Invalid)?;
        refusal(switches, module, on).map_or(Ok(()), Err)?;
        let target = self.target(&snapshot);
        let preparers: Vec<Arc<dyn ModulePreparer>> = self
            .preparers
            .iter()
            .filter(|preparer| on && preparer.module() == module)
            .filter(|preparer| {
                preparer.touches().iter().all(|section| {
                    snapshot
                        .origins
                        .table(section)
                        .is_none_or(|origin| origin == target)
                })
            })
            .cloned()
            .collect();
        let (_, written) = self
            .configuration
            .update(&target, revision, |document| {
                write_switch(document, module, on);
                for preparer in &preparers {
                    preparer.prepare(document);
                }
                Ok(())
            })
            .await?;
        let switches = ModuleSwitches::resolve(&written.document).map_err(ApiError::Invalid)?;
        Ok(Revisioned::new(views(switches), written.revision))
    }

    fn target(&self, snapshot: &Snapshot) -> PathBuf {
        snapshot
            .origins
            .table(SECTION)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.configuration.path().to_path_buf())
    }
}

fn refusal(switches: ModuleSwitches, module: Module, on: bool) -> Option<ApiError> {
    if on {
        let missing = switches.missing_requirements(module);
        return (!missing.is_empty()).then(|| {
            let names = listed(&missing);
            ApiError::Conflict(format!(
                "{} needs {names}, which is off; switch {names} on first",
                module.name()
            ))
        });
    }
    let needing = switches.required_by(module);
    (!needing.is_empty()).then(|| {
        let names = listed(&needing);
        ApiError::Conflict(format!(
            "{names} needs {}; switch {names} off first",
            module.name()
        ))
    })
}

fn listed(modules: &[Module]) -> String {
    modules
        .iter()
        .map(|module| module.name())
        .collect::<Vec<_>>()
        .join(", ")
}
