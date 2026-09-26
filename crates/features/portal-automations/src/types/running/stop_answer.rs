#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopAnswer {
    Stopping,
    AlreadyStopping,
    Finished,
    Unknown,
}
