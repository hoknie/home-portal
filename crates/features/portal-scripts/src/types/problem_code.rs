use portal_model::ScriptPathProblem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemCode {
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

impl ProblemCode {
    pub fn name(self) -> &'static str {
        match self {
            ProblemCode::Shape => "shape",
            ProblemCode::TooDeep => "too-deep",
            ProblemCode::Hidden => "hidden",
            ProblemCode::NotFound => "not-found",
            ProblemCode::Outside => "outside",
            ProblemCode::NotAFile => "not-a-file",
            ProblemCode::NotExecutable => "not-executable",
            ProblemCode::Writable => "writable",
            ProblemCode::Owner => "owner",
            ProblemCode::FolderWritable => "folder-writable",
            ProblemCode::FolderOwner => "folder-owner",
        }
    }

    pub fn of_shape(problem: ScriptPathProblem) -> ProblemCode {
        match problem {
            ScriptPathProblem::Hidden => ProblemCode::Hidden,
            ScriptPathProblem::TooDeep => ProblemCode::TooDeep,
            _ => ProblemCode::Shape,
        }
    }
}
