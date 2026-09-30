use std::collections::HashMap;

use axum::http::Method;
use portal_feature::{Requirement, Rule};

#[derive(Debug, Clone, Default)]
pub struct RuleBook {
    rules: HashMap<(Method, &'static str), Requirement>,
}

impl RuleBook {
    pub fn of(rules: impl IntoIterator<Item = Rule>) -> RuleBook {
        RuleBook {
            rules: rules
                .into_iter()
                .map(|rule| ((rule.method, rule.path), rule.requirement))
                .collect(),
        }
    }

    pub fn requirement(&self, method: &Method, path: &str) -> Option<Requirement> {
        self.rules.get(&(method.clone(), path)).copied()
    }

    pub fn rules(&self) -> Vec<(Method, &'static str, Requirement)> {
        self.rules
            .iter()
            .map(|((method, path), requirement)| (method.clone(), *path, *requirement))
            .collect()
    }
}
