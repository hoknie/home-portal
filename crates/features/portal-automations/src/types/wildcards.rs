#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Wildcards {
    pub hour: bool,
    pub day: bool,
    pub weekday: bool,
}
