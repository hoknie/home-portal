#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalCode {
    Shape,
    TooDeep,
    Hidden,
    NotFound,
    Outside,
    NotAFile,
    NotExecutable,
    Writable,
    Owner,
    FolderWritable,
    FolderOwner,
}

impl RefusalCode {
    pub fn name(self) -> &'static str {
        match self {
            RefusalCode::Shape => "shape",
            RefusalCode::TooDeep => "too-deep",
            RefusalCode::Hidden => "hidden",
            RefusalCode::NotFound => "not-found",
            RefusalCode::Outside => "outside",
            RefusalCode::NotAFile => "not-a-file",
            RefusalCode::NotExecutable => "not-executable",
            RefusalCode::Writable => "writable",
            RefusalCode::Owner => "owner",
            RefusalCode::FolderWritable => "folder-writable",
            RefusalCode::FolderOwner => "folder-owner",
        }
    }
}
