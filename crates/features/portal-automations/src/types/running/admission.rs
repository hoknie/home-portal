use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct Admission {
    pub running: bool,
    pub pending: bool,
    pub last_start: Option<Instant>,
    pub starts: VecDeque<Instant>,
}
