use axum::http::Method;

use super::{Requirement, Right};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub method: Method,
    pub path: &'static str,
    pub requirement: Requirement,
}

impl Rule {
    pub fn signed(method: Method, path: &'static str) -> Rule {
        Rule {
            method,
            path,
            requirement: Requirement::Signed,
        }
    }

    pub fn needs(method: Method, path: &'static str, rights: &'static [Right]) -> Rule {
        Rule {
            method,
            path,
            requirement: Requirement::AnyOf(rights),
        }
    }

    pub fn admin(method: Method, path: &'static str) -> Rule {
        Rule {
            method,
            path,
            requirement: Requirement::Admin,
        }
    }
}
