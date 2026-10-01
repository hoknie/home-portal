use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use portal_config::{ConfigError, ConfigStore, Section};
use toml_edit::{DocumentMut, Item, Table};

pub const FILES_SECTION: &str = "files";
pub const PRIVATE_MODE: u32 = 0o600;
pub const WORKFLOW_ID: &str = "id";

fn home_of(main: &Path, document: &DocumentMut, section: Section) -> PathBuf {
    let base = main.parent().unwrap_or(Path::new("."));
    let chosen = document
        .get(FILES_SECTION)
        .and_then(Item::as_table_like)
        .and_then(|table| table.get(section.key()))
        .and_then(Item::as_str)
        .map(str::to_string);
    base.join(chosen.unwrap_or_else(|| section.default_home().to_string()))
}

fn appended(path: &Path, key: &str, item: Item) {
    let mut document: DocumentMut = fs::read_to_string(path)
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    match (document.get_mut(key), item) {
        (Some(Item::ArrayOfTables(entries)), Item::ArrayOfTables(added)) => {
            for entry in added.iter() {
                entries.push(entry.clone());
            }
        }
        (_, item) => {
            document.insert(key, item);
        }
    }
    write_private(path, &document.to_string());
}

fn write_private(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("the home's folder can be created");
    }
    fs::write(path, text).expect("a home can be written");
    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_MODE))
        .expect("a home can be made private");
}

fn workflow_files(folder: &Path, item: &Item) {
    for entry in item
        .as_array_of_tables()
        .into_iter()
        .flat_map(|entries| entries.iter())
    {
        let id = entry
            .get(WORKFLOW_ID)
            .and_then(Item::as_str)
            .expect("a workflow in a fixture has an id")
            .to_string();
        let mut table: Table = entry.clone();
        let comment = table
            .decor()
            .prefix()
            .and_then(|prefix| prefix.as_str())
            .map(|prefix| prefix.trim_start_matches(['\n', '\r']).to_string())
            .unwrap_or_default();
        table.decor_mut().clear();
        table.decor_mut().set_prefix(comment);
        table.set_implicit(false);
        table.set_position(None);
        write_private(
            &folder.join(format!("{id}.toml")),
            &DocumentMut::from(table).to_string(),
        );
    }
}

pub fn split(main: &Path) {
    let text = fs::read_to_string(main).expect("the main file can be read");
    let mut document: DocumentMut = text.parse().expect("the main file is TOML");
    let keys: Vec<String> = document.iter().map(|(key, _)| key.to_string()).collect();
    let mut moved = false;
    for key in keys {
        let Some(section) = Section::of_table(&key) else {
            continue;
        };
        let home = home_of(main, &document, section);
        let item = document.remove(&key).expect("the key was listed");
        if section.is_folder() {
            workflow_files(&home, &item);
        } else {
            appended(&home, &key, item);
        }
        moved = true;
    }
    if moved {
        fs::write(main, document.to_string()).expect("the main file can be written");
    }
}

pub fn opened(main: impl Into<PathBuf>) -> Result<ConfigStore, ConfigError> {
    let main = main.into();
    split(&main);
    ConfigStore::open(main)
}
