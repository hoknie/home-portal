use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, Table};

use crate::helpers::{
    create_private_folder, include_paths, next_position, parse_document, root_of, shifted,
    write_atomically,
};
use crate::types::{ConfigError, Layout, Section};

struct Planned {
    document: DocumentMut,
    bytes: Vec<u8>,
    gained: bool,
    lost: bool,
}

struct Plan {
    files: BTreeMap<PathBuf, Planned>,
    order: Vec<PathBuf>,
    workflows: Vec<(PathBuf, String)>,
    moves: Vec<(String, PathBuf, PathBuf)>,
}

pub fn settle(main: &Path, layout: &Layout) -> Result<(), ConfigError> {
    let Some(plan) = planned(main, layout)? else {
        return Ok(());
    };
    written(plan, layout)
}

pub fn pending_moves(main: &Path) -> Result<Vec<(String, PathBuf, PathBuf)>, ConfigError> {
    let Some((document, _)) = read(main) else {
        return Ok(Vec::new());
    };
    let layout = crate::helpers::layout_of(main, &document);
    Ok(planned(main, &layout)?
        .map(|plan| plan.moves)
        .unwrap_or_default())
}

fn read(path: &Path) -> Option<(DocumentMut, Vec<u8>)> {
    let bytes = fs::read(path).ok()?;
    parse_document(&bytes)
        .ok()
        .map(|document| (document, bytes))
}

fn planned(main: &Path, layout: &Layout) -> Result<Option<Plan>, ConfigError> {
    let Some((main_document, main_bytes)) = read(main) else {
        return Ok(None);
    };
    let Ok(includes) = include_paths(main, &main_document) else {
        return Ok(None);
    };
    let mut plan = Plan {
        files: BTreeMap::new(),
        order: Vec::new(),
        workflows: Vec::new(),
        moves: Vec::new(),
    };
    plan.add(main, main_document, main_bytes);
    for path in includes.iter().chain(layout.files().iter()) {
        if plan.files.contains_key(path) || path.as_path() == main {
            continue;
        }
        if let Some((document, bytes)) = read(path) {
            plan.add(path, document, bytes);
        } else if path.exists() {
            return Ok(None);
        }
    }
    for path in plan.order.clone() {
        plan.settle_file(&path, layout)?;
    }
    Ok((!plan.moves.is_empty()).then_some(plan))
}

impl Plan {
    fn add(&mut self, path: &Path, document: DocumentMut, bytes: Vec<u8>) {
        self.order.push(path.to_path_buf());
        self.files.insert(
            path.to_path_buf(),
            Planned {
                document,
                bytes,
                gained: false,
                lost: false,
            },
        );
    }

    fn target(&mut self, path: &Path) -> &mut Planned {
        if !self.files.contains_key(path) {
            self.add(path, DocumentMut::new(), Vec::new());
        }
        self.files.get_mut(path).expect("just added")
    }

    fn settle_file(&mut self, path: &Path, layout: &Layout) -> Result<(), ConfigError> {
        let keys: Vec<String> = self.files[path]
            .document
            .iter()
            .map(|(key, _)| key.to_string())
            .collect();
        for key in keys {
            let Some(section) = Section::of_table(&key) else {
                continue;
            };
            if section == Section::Workflows {
                self.settle_workflows(path, layout)?;
                continue;
            }
            let home = layout.file_of(section);
            if home == path {
                continue;
            }
            let item = self
                .files
                .get_mut(path)
                .expect("scanned")
                .document
                .remove(&key)
                .expect("listed");
            let target = self.target(&home);
            let next = next_position(&target.document);
            merge_item(target.document.as_table_mut(), &key, shifted(item, next))
                .map_err(|dotted| collision(&dotted, path, &home))?;
            target.gained = true;
            self.files.get_mut(path).expect("scanned").lost = true;
            self.moves.push((key, path.to_path_buf(), home));
        }
        Ok(())
    }

    fn settle_workflows(&mut self, path: &Path, layout: &Layout) -> Result<(), ConfigError> {
        let key = Section::Workflows.key();
        let entries = match self.files[path].document.get(key) {
            Some(Item::ArrayOfTables(entries)) => entries.clone(),
            _ => return Err(collision(key, path, &layout.folder())),
        };
        let mut planned = Vec::new();
        for entry in entries.iter() {
            let id = entry
                .get("id")
                .and_then(Item::as_str)
                .unwrap_or_default()
                .to_string();
            if !valid_id(&id) {
                return Err(ConfigError::Merge {
                    message: format!(
                        "a workflow in {} has no valid id, so it cannot be moved to {}; give it an id or move it by hand",
                        path.display(),
                        layout.folder().display()
                    ),
                });
            }
            let file = layout.workflow_file(&id);
            if file.exists()
                || self
                    .workflows
                    .iter()
                    .chain(planned.iter())
                    .any(|(taken, _)| *taken == file)
            {
                return Err(collision(&format!("{key} {id}"), path, &file));
            }
            planned.push((file, root_of(entry.clone()).to_string()));
        }
        self.files
            .get_mut(path)
            .expect("scanned")
            .document
            .remove(key);
        self.files.get_mut(path).expect("scanned").lost = true;
        for (file, text) in planned {
            self.moves
                .push((key.to_string(), path.to_path_buf(), file.clone()));
            self.workflows.push((file, text));
        }
        Ok(())
    }
}

fn written(plan: Plan, layout: &Layout) -> Result<(), ConfigError> {
    let failure = |path: &Path, error: std::io::Error| ConfigError::Unreadable {
        path: path.to_path_buf(),
        source: error,
    };
    if !plan.workflows.is_empty() {
        let folder = layout.folder();
        create_private_folder(&folder).map_err(|error| failure(&folder, error))?;
    }
    for (file, text) in &plan.workflows {
        write_atomically(file, &[], text).map_err(|error| failure(file, error))?;
    }
    for lost in [false, true] {
        for path in &plan.order {
            let planned = &plan.files[path];
            if (planned.gained && !lost) || (planned.lost && !planned.gained && lost) {
                if let Some(parent) = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                {
                    create_private_folder(parent).map_err(|error| failure(parent, error))?;
                }
                let text = planned.document.to_string();
                let text = if planned.bytes.is_empty() {
                    text.trim_start().to_string()
                } else {
                    text
                };
                write_atomically(path, &planned.bytes, &text)
                    .map_err(|error| failure(path, error))?;
            }
        }
    }
    for (section, from, to) in &plan.moves {
        tracing::info!(section = %section, from = %from.display(), to = %to.display(), "configuration moved to its home");
    }
    Ok(())
}

fn collision(dotted: &str, from: &Path, home: &Path) -> ConfigError {
    ConfigError::Merge {
        message: format!(
            "{dotted} is in {} and already in {}; merge them by hand, then start again",
            from.display(),
            home.display()
        ),
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

fn merge_item(into: &mut Table, key: &str, item: Item) -> Result<(), String> {
    match (into.get_mut(key), item) {
        (None, item) => {
            into.insert(key, item);
            Ok(())
        }
        (Some(Item::ArrayOfTables(existing)), Item::ArrayOfTables(entries)) => {
            for entry in entries {
                existing.push(entry);
            }
            Ok(())
        }
        (Some(Item::Table(existing)), Item::Table(table)) => {
            for (child, value) in table {
                merge_item(existing, &child, value).map_err(|dotted| format!("{key}.{dotted}"))?;
            }
            Ok(())
        }
        _ => Err(key.to_string()),
    }
}
