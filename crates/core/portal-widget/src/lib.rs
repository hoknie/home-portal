mod controllers;
mod services;
mod types;

pub use controllers::{DATA_PATH, answered, widgets_router};
pub use services::WidgetRegistry;
pub use types::{
    Accent, Align, Layout, Padding, ResolvedAppearance, ResolvedSectionAppearance,
    SectionAppearance, SectionEntry, SectionSurface, Surface, TitleVisibility, WidgetAnswer,
    WidgetAppearance, WidgetData, WidgetHeight, WidgetInstance, WidgetSize, WidgetsSection,
};
