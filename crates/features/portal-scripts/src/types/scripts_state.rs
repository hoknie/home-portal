use crate::usecases::{
    CreateFolder, CreateScript, DeleteFolder, DeleteScript, ListScripts, MoveScript, ReadScript,
    ScriptEditing, WriteScript,
};

#[derive(Clone)]
pub struct ScriptsState {
    pub editing: ScriptEditing,
    pub list: ListScripts,
    pub read: ReadScript,
    pub write: WriteScript,
    pub create: CreateScript,
    pub delete: DeleteScript,
    pub create_folder: CreateFolder,
    pub delete_folder: DeleteFolder,
    pub move_script: MoveScript,
}
