#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryRange {
    Day,
    Week,
    Month,
}

impl HistoryRange {
    pub const ALL: [HistoryRange; 3] = [HistoryRange::Day, HistoryRange::Week, HistoryRange::Month];

    pub fn parse(name: &str) -> Option<HistoryRange> {
        HistoryRange::ALL
            .into_iter()
            .find(|range| range.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            HistoryRange::Day => "24h",
            HistoryRange::Week => "7d",
            HistoryRange::Month => "30d",
        }
    }

    pub fn seconds(self) -> i64 {
        match self {
            HistoryRange::Day => 86_400,
            HistoryRange::Week => 7 * 86_400,
            HistoryRange::Month => 30 * 86_400,
        }
    }
}
