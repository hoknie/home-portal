use portal_model::ProbeOutcome;

use super::ProbeKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeReport {
    pub kind: ProbeKind,
    pub target: String,
    pub outcome: ProbeOutcome,
}
