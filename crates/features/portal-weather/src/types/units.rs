use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    #[default]
    Metric,
    Imperial,
}

impl Units {
    pub fn temperature(&self) -> &'static str {
        match self {
            Units::Metric => "celsius",
            Units::Imperial => "fahrenheit",
        }
    }

    pub fn wind(&self) -> &'static str {
        match self {
            Units::Metric => "kmh",
            Units::Imperial => "mph",
        }
    }
}
