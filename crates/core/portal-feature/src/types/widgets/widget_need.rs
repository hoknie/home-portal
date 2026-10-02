use crate::types::Right;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedScope {
    SameWidget,
    AnyWidget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetNeed {
    pub field: String,
    pub right: Right,
    pub fingerprint: String,
    pub scope: NeedScope,
}

impl WidgetNeed {
    pub fn new(
        field: impl Into<String>,
        right: Right,
        fingerprint: impl Into<String>,
        scope: NeedScope,
    ) -> WidgetNeed {
        WidgetNeed {
            field: field.into(),
            right,
            fingerprint: fingerprint.into(),
            scope,
        }
    }
}
