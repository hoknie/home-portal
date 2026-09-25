use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WidgetSize {
    Quarter,
    Third,
    Half,
    TwoThirds,
    #[default]
    Full,
    #[serde(other)]
    Unknown,
}

impl WidgetSize {
    pub const ALL: [WidgetSize; 5] = [
        WidgetSize::Quarter,
        WidgetSize::Third,
        WidgetSize::Half,
        WidgetSize::TwoThirds,
        WidgetSize::Full,
    ];
    pub const NAMES: &'static str = "quarter, third, half, two-thirds, full";

    pub fn name(self) -> &'static str {
        match self {
            WidgetSize::Quarter => "quarter",
            WidgetSize::Third => "third",
            WidgetSize::Half => "half",
            WidgetSize::TwoThirds => "two-thirds",
            WidgetSize::Full => "full",
            WidgetSize::Unknown => "unknown",
        }
    }

    pub fn columns(self) -> u8 {
        match self {
            WidgetSize::Quarter => 3,
            WidgetSize::Third => 4,
            WidgetSize::Half => 6,
            WidgetSize::TwoThirds => 8,
            WidgetSize::Full | WidgetSize::Unknown => 12,
        }
    }
}
