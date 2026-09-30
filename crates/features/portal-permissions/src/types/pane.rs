use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Pane {
    LocalNetwork,
    FilesAndFolders,
    Automation,
    FullDiskAccess,
}

impl Pane {
    pub fn title(self) -> &'static str {
        match self {
            Pane::LocalNetwork => "Privacy & Security > Local Network",
            Pane::FilesAndFolders => "Privacy & Security > Files and Folders",
            Pane::Automation => "Privacy & Security > Automation",
            Pane::FullDiskAccess => "Privacy & Security > Full Disk Access",
        }
    }
}
