use axum::Router;

use std::sync::Arc;

use crate::ports::WidgetProvider;
use crate::types::{Loop, Rule, Validator};

pub trait Feature: Send + Sync {
    fn name(&self) -> &'static str;

    fn router(&self) -> Router;

    fn public_router(&self) -> Router {
        Router::new()
    }

    fn validator(&self) -> Option<Validator> {
        None
    }

    fn rules(&self) -> Vec<Rule> {
        Vec::new()
    }

    fn loops(&self) -> Vec<Loop> {
        Vec::new()
    }

    fn widget_providers(&self) -> Vec<Arc<dyn WidgetProvider>> {
        Vec::new()
    }

    fn stop(&self) {}
}
