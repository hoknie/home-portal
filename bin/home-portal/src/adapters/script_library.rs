use std::path::PathBuf;

use portal_automations::{Refusal, RefusalCode, ScriptEntry, ScriptLibrary};
use portal_scripts::{ListScripts, ProblemCode, ResolveScript, ScriptEditing, ScriptProblem};

pub struct ScriptShelf {
    pub resolve: ResolveScript,
    pub list: ListScripts,
    pub editing: ScriptEditing,
}

impl ScriptLibrary for ScriptShelf {
    fn root(&self) -> PathBuf {
        self.list.root()
    }

    fn resolve(&self, script: &str) -> Result<PathBuf, Refusal> {
        self.resolve.run(script).map_err(refusal)
    }

    fn list(&self) -> Option<Vec<ScriptEntry>> {
        self.list.run().map(|files| {
            files
                .into_iter()
                .map(|file| ScriptEntry {
                    path: file.path,
                    problem: file.problem.map(refusal),
                    header: file.header,
                })
                .collect()
        })
    }

    fn editing(&self) -> bool {
        self.editing.run()
    }
}

fn refusal(problem: ScriptProblem) -> Refusal {
    Refusal {
        code: code(problem.code),
        path: problem.path,
        message: problem.message,
    }
}

fn code(code: ProblemCode) -> RefusalCode {
    match code {
        ProblemCode::Shape => RefusalCode::Shape,
        ProblemCode::TooDeep => RefusalCode::TooDeep,
        ProblemCode::Hidden => RefusalCode::Hidden,
        ProblemCode::NotFound => RefusalCode::NotFound,
        ProblemCode::Outside => RefusalCode::Outside,
        ProblemCode::NotAFile => RefusalCode::NotAFile,
        ProblemCode::NotExecutable => RefusalCode::NotExecutable,
        ProblemCode::Writable => RefusalCode::Writable,
        ProblemCode::Owner => RefusalCode::Owner,
        ProblemCode::FolderWritable => RefusalCode::FolderWritable,
        ProblemCode::FolderOwner => RefusalCode::FolderOwner,
    }
}
