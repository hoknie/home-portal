#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ending {
    Succeeded(Option<String>),
    Failed(String),
    TimedOut,
    Stopped,
}
