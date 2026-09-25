use portal_feature::FieldError;
use url::Url;

use crate::types::{ServiceEntry, ServiceLink};

pub fn check_links(entry: &ServiceEntry) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if entry.links.len() > ServiceEntry::MAXIMUM_LINKS {
        errors.push(FieldError::new(
            "links",
            format!("must hold at most {} links", ServiceEntry::MAXIMUM_LINKS),
        ));
    }
    for (index, link) in entry.links.iter().enumerate() {
        let title = link.title.trim();
        if title.is_empty() || title.chars().count() > ServiceLink::MAXIMUM_TITLE_LENGTH {
            errors.push(FieldError::new(
                format!("links[{index}].title"),
                format!(
                    "must be 1 to {} characters",
                    ServiceLink::MAXIMUM_TITLE_LENGTH
                ),
            ));
        }
        let accepted = Url::parse(link.url.trim()).is_ok_and(|url| {
            matches!(url.scheme(), "http" | "https")
                && url.host_str().is_some_and(|host| !host.is_empty())
        });
        if !accepted {
            errors.push(FieldError::new(
                format!("links[{index}].url"),
                "must be an absolute http or https URL",
            ));
        }
    }
    errors
}

pub fn check_notes(entry: &ServiceEntry) -> Vec<FieldError> {
    match &entry.notes {
        Some(notes) if notes.chars().count() > ServiceEntry::MAXIMUM_NOTES_LENGTH => {
            vec![FieldError::new(
                "notes",
                format!(
                    "must be at most {} characters",
                    ServiceEntry::MAXIMUM_NOTES_LENGTH
                ),
            )]
        }
        _ => Vec::new(),
    }
}

pub fn check_widgets(entry: &ServiceEntry, known: &[String]) -> Vec<FieldError> {
    entry
        .widgets
        .iter()
        .enumerate()
        .filter(|(_, id)| !known.contains(id))
        .map(|(index, id)| {
            FieldError::new(
                format!("widgets[{index}]"),
                format!("names {id}, and no widget has that id"),
            )
        })
        .collect()
}
