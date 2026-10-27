#[derive(Debug, Clone, PartialEq)]
pub struct PlannedEntry<T> {
    pub stored: Option<usize>,
    pub old: T,
    pub wanted: T,
}
