use crate::usecases::{PressWidgetAction, PreviewWidget};

#[derive(Clone)]
pub struct WidgetsState {
    pub press: PressWidgetAction,
    pub preview: PreviewWidget,
}
