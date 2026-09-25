use std::path::Path;

use toml_edit::{DocumentMut, Item, Table};

use crate::types::{ConfigError, Origins, Source};

use super::includes::INCLUDE_KEY;

pub fn merge(sources: &[Source]) -> Result<(DocumentMut, Origins), ConfigError> {
    let mut merged = DocumentMut::new();
    let mut origins = Origins::default();
    for source in sources {
        merge_table(
            merged.as_table_mut(),
            source.document.as_table(),
            &source.path,
            "",
            &mut origins,
            sources,
        )?;
    }
    merged.remove(INCLUDE_KEY);
    Ok((merged, origins))
}

fn merge_table(
    into: &mut Table,
    from: &Table,
    path: &Path,
    prefix: &str,
    origins: &mut Origins,
    sources: &[Source],
) -> Result<(), ConfigError> {
    for (key, item) in from.iter() {
        let dotted = if prefix.is_empty() {
            key.to_string()
        } else {
            format!("{prefix}.{key}")
        };
        match (into.get_mut(key), item) {
            (None, Item::ArrayOfTables(entries)) => {
                for _ in entries.iter() {
                    origins.record(&dotted, path);
                }
                into.insert(key, item.clone());
            }
            (None, Item::Table(table)) => {
                origins.record_table(&dotted, path);
                into.insert(key, Item::Table(Table::new()));
                let child = into[key].as_table_mut().expect("just inserted a table");
                child.set_implicit(table.is_implicit());
                merge_table(child, table, path, &dotted, origins, sources)?;
            }
            (None, _) => {
                origins.record_table(&dotted, path);
                into.insert(key, item.clone());
            }
            (Some(Item::ArrayOfTables(existing)), Item::ArrayOfTables(entries)) => {
                for entry in entries.iter() {
                    existing.push(entry.clone());
                    origins.record(&dotted, path);
                }
            }
            (Some(Item::Table(_)), Item::Table(table)) => {
                let child = into[key].as_table_mut().expect("matched a table");
                merge_table(child, table, path, &dotted, origins, sources)?;
            }
            (Some(_), _) => {
                return Err(ConfigError::Merge {
                    message: format!(
                        "{dotted} is set by {} and by {}",
                        first_holder(sources, &dotted, path),
                        path.display()
                    ),
                });
            }
        }
    }
    Ok(())
}

fn first_holder(sources: &[Source], dotted: &str, current: &Path) -> String {
    sources
        .iter()
        .filter(|source| source.path != current)
        .find(|source| holds(source.document.as_table(), dotted))
        .map(|source| source.path.display().to_string())
        .unwrap_or_else(|| "another file".to_string())
}

fn holds(table: &Table, dotted: &str) -> bool {
    match dotted.split_once('.') {
        None => table.contains_key(dotted),
        Some((head, rest)) => table
            .get(head)
            .and_then(Item::as_table)
            .is_some_and(|child| holds(child, rest)),
    }
}
