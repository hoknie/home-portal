use std::path::{Component, Path};

use super::ScriptPathProblem;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScriptPath {
    parts: Vec<String>,
}

impl ScriptPath {
    pub const DEEPEST: usize = 2;

    pub fn parse(text: &str) -> Result<ScriptPath, ScriptPathProblem> {
        if text.trim().is_empty() {
            return Err(ScriptPathProblem::Empty);
        }
        let path = Path::new(text);
        if path.is_absolute() {
            return Err(ScriptPathProblem::Absolute);
        }
        let mut parts = Vec::new();
        for component in path.components() {
            match component {
                Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
                Component::CurDir => {}
                _ => return Err(ScriptPathProblem::Climbs),
            }
        }
        if parts.is_empty() {
            return Err(ScriptPathProblem::Empty);
        }
        if parts.iter().any(|part| part.starts_with('.')) {
            return Err(ScriptPathProblem::Hidden);
        }
        if parts.len() > Self::DEEPEST {
            return Err(ScriptPathProblem::TooDeep);
        }
        Ok(ScriptPath { parts })
    }

    pub fn problem(text: &str) -> Option<&'static str> {
        Self::parse(text).err().map(ScriptPathProblem::message)
    }

    pub fn text(&self) -> String {
        self.parts.join("/")
    }

    pub fn name(&self) -> &str {
        self.parts.last().map(String::as_str).unwrap_or_default()
    }

    pub fn folder(&self) -> Option<&str> {
        (self.parts.len() > 1).then(|| self.parts[0].as_str())
    }
}
