use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetLimits {
    pub refresh: Duration,
    pub timeout: Duration,
}

impl WidgetLimits {
    pub const LONGEST_WAIT: Duration = Duration::from_secs(15);

    pub fn of(refresh: Duration) -> WidgetLimits {
        WidgetLimits {
            refresh,
            timeout: refresh.min(Self::LONGEST_WAIT),
        }
    }
}
