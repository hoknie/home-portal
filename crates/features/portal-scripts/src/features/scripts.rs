use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use portal_config::ConfigStore;
use portal_feature::{Feature, Validator};

use crate::controllers::{
    create, create_folder, move_script, read, remove, remove_folder, replace, tree,
};
use crate::ports::ProcessIdentity;
use crate::services::validate_scripts;
use crate::types::ScriptsState;
use crate::usecases::{
    CreateFolder, CreateScript, DeleteFolder, DeleteScript, MoveScript, ReadScript, ScriptEditing,
    ScriptFiles, WriteScript,
};

pub struct ScriptsFeature {
    state: ScriptsState,
    files: ScriptFiles,
}

impl ScriptsFeature {
    pub const NAME: &'static str = "scripts";
    pub const TREE: &'static str = "/api/scripts";
    pub const FILE: &'static str = "/api/scripts/file";
    pub const FOLDER: &'static str = "/api/scripts/folder";
    pub const MOVE: &'static str = "/api/scripts/move";
    pub const LARGEST_BODY: usize = 512 * 1024;

    pub fn new(
        configuration: Arc<ConfigStore>,
        identity: Arc<dyn ProcessIdentity>,
    ) -> ScriptsFeature {
        let files = ScriptFiles::new(&configuration, identity);
        ScriptsFeature {
            state: ScriptsState {
                editing: ScriptEditing::new(configuration),
                list: files.list.clone(),
                read: ReadScript::new(files.clone()),
                write: WriteScript::new(files.clone()),
                create: CreateScript::new(files.clone()),
                delete: DeleteScript::new(files.clone()),
                create_folder: CreateFolder::new(files.clone()),
                delete_folder: DeleteFolder::new(files.clone()),
                move_script: MoveScript::new(files.clone()),
            },
            files,
        }
    }
}

impl Feature for ScriptsFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        self.files.writer.sweep();
        Router::new()
            .route(Self::TREE, get(tree))
            .route(
                Self::FILE,
                get(read).put(replace).post(create).delete(remove),
            )
            .route(Self::FOLDER, post(create_folder).delete(remove_folder))
            .route(Self::MOVE, post(move_script))
            .layer(axum::extract::DefaultBodyLimit::max(Self::LARGEST_BODY))
            .with_state(self.state.clone())
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_scripts)
    }
}
