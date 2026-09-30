use std::env;
use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OwnerKind {
    Terminal,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owner {
    pub kind: OwnerKind,
    pub name: String,
}

impl Owner {
    pub const TERMINAL_VARIABLE: &'static str = "TERM_PROGRAM";
    pub const SOME_TERMINAL: &'static str = "the terminal application";
    pub const SOME_BINARY: &'static str = "home-portal";

    pub fn detect(terminal: Option<String>, executable: Option<PathBuf>) -> Owner {
        match terminal.filter(|name| !name.is_empty()) {
            Some(name) => Owner {
                kind: OwnerKind::Terminal,
                name: terminal_name(&name),
            },
            None => Owner {
                kind: OwnerKind::Binary,
                name: executable.map_or_else(
                    || Self::SOME_BINARY.to_string(),
                    |path| path.display().to_string(),
                ),
            },
        }
    }

    pub fn of_this_process() -> Owner {
        Owner::detect(
            env::var(Self::TERMINAL_VARIABLE).ok(),
            env::current_exe().ok(),
        )
    }
}

fn terminal_name(program: &str) -> String {
    match program {
        "Apple_Terminal" => "Terminal".to_string(),
        "iTerm.app" => "iTerm".to_string(),
        "vscode" => "Visual Studio Code".to_string(),
        other => other.to_string(),
    }
}
