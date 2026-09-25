use portal_widget::WidgetInstance;

#[derive(Debug, Clone, PartialEq)]
pub struct EditedWidget {
    pub key: Option<String>,
    pub instance: WidgetInstance,
}
