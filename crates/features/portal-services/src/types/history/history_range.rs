#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryRange {
    Hour,
    SixHours,
    Day,
    Week,
    Month,
}

impl HistoryRange {
    pub const ALL: [HistoryRange; 5] = [
        HistoryRange::Hour,
        HistoryRange::SixHours,
        HistoryRange::Day,
        HistoryRange::Week,
        HistoryRange::Month,
    ];
    pub const UPTIME: [HistoryRange; 3] =
        [HistoryRange::Day, HistoryRange::Week, HistoryRange::Month];

    pub fn parse(name: &str) -> Option<HistoryRange> {
        HistoryRange::ALL
            .into_iter()
            .find(|range| range.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            HistoryRange::Hour => "1h",
            HistoryRange::SixHours => "6h",
            HistoryRange::Day => "24h",
            HistoryRange::Week => "7d",
            HistoryRange::Month => "30d",
        }
    }

    pub fn seconds(self) -> i64 {
        match self {
            HistoryRange::Hour => 3_600,
            HistoryRange::SixHours => 6 * 3_600,
            HistoryRange::Day => 86_400,
            HistoryRange::Week => 7 * 86_400,
            HistoryRange::Month => 30 * 86_400,
        }
    }

    pub fn step_seconds(self) -> i64 {
        match self {
            HistoryRange::Hour => 60,
            HistoryRange::SixHours => 5 * 60,
            HistoryRange::Day => 15 * 60,
            HistoryRange::Week => 2 * 3_600,
            HistoryRange::Month => 6 * 3_600,
        }
    }

    pub fn from_samples(self) -> bool {
        matches!(
            self,
            HistoryRange::Hour | HistoryRange::SixHours | HistoryRange::Day
        )
    }
}
