use super::InlinePlacement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredWidgets {
    pub dashboard: bool,
    pub library_ids: Vec<String>,
    pub inline: Vec<InlinePlacement>,
}
