use crate::types::Choice;

pub trait Directory: Send + Sync {
    fn services(&self) -> Vec<Choice>;

    fn users(&self) -> Vec<String>;

    fn environments(&self) -> Vec<String>;
}
