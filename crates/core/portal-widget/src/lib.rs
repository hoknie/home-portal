mod controllers;
mod services;
mod types;

pub use controllers::widgets_router;
pub use services::WidgetRegistry;
pub use types::{Layout, SectionEntry, WidgetData, WidgetInstance, WidgetSize, WidgetsSection};
