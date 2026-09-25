use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wake {
    Woken,
    NotFound,
    Disabled,
    TooSoon(Duration),
}
