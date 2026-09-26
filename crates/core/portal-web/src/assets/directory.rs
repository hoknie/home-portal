use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use crate::helpers::{FALLBACK_CONTENT_TYPE, content_type_of};
use crate::ports::AssetSource;
use crate::types::Asset;

pub const ENTRY_PAGES: [&str; 2] = ["index.html", "en/index.html"];
pub const WEB_VARIABLE: &str = "HOME_PORTAL_WEB";

#[derive(Debug, Clone)]
pub struct Directory {
    root: Option<PathBuf>,
    searched: Vec<PathBuf>,
}

impl Directory {
    pub fn first_of(candidates: &[PathBuf]) -> Directory {
        let root = candidates
            .iter()
            .find(|candidate| Self::holds_interface(candidate))
            .and_then(|candidate| fs::canonicalize(candidate).ok());
        Directory {
            root,
            searched: candidates.to_vec(),
        }
    }

    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    fn holds_interface(folder: &Path) -> bool {
        ENTRY_PAGES.iter().any(|page| folder.join(page).is_file())
    }

    fn inside(&self, path: &str) -> Option<PathBuf> {
        let root = self.root.as_ref()?;
        let safe = path.split('/').all(|segment| {
            !segment.is_empty() && segment != "." && segment != ".." && !segment.contains('\\')
        });
        if !safe {
            return None;
        }
        let found = fs::canonicalize(root.join(path)).ok()?;
        (found.starts_with(root) && found.is_file()).then_some(found)
    }
}

impl AssetSource for Directory {
    fn get(&self, path: &str) -> Option<Asset> {
        let file = self.inside(path)?;
        let bytes = fs::read(&file).ok()?;
        Some(Asset {
            bytes: Cow::Owned(bytes),
            content_type: content_type_of(path)
                .unwrap_or(FALLBACK_CONTENT_TYPE)
                .to_string(),
        })
    }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    fn unavailable(&self) -> String {
        let searched: Vec<String> = self
            .searched
            .iter()
            .map(|place| place.display().to_string())
            .collect();
        format!(
            "the interface files were not found; looked in {}; set {WEB_VARIABLE} to the folder that holds them",
            if searched.is_empty() {
                "no place".to_string()
            } else {
                searched.join(", ")
            }
        )
    }
}
