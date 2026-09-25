use serde::{Deserialize, Serialize};

use super::{HourBucket, Sample, Transition};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HistoryLine {
    Sample(Sample),
    Replace(Sample),
    Transition(Transition),
    Hour(HourBucket),
}
