use crate::types::Revision;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revisioned<T> {
    pub value: T,
    pub revision: Revision,
}

impl<T> Revisioned<T> {
    pub fn new(value: T, revision: Revision) -> Revisioned<T> {
        Revisioned { value, revision }
    }

    pub fn map<U>(self, change: impl FnOnce(T) -> U) -> Revisioned<U> {
        Revisioned {
            value: change(self.value),
            revision: self.revision,
        }
    }
}
