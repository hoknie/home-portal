#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Place {
    pub path: String,
    pub iteration: Option<usize>,
    pub calls: usize,
}

impl Place {
    pub fn root() -> Place {
        Place {
            path: "steps".to_string(),
            iteration: None,
            calls: 0,
        }
    }

    pub fn inside(&self, list: &str) -> Place {
        Place {
            path: format!("{}.{list}", self.path),
            ..self.clone()
        }
    }

    pub fn repeated(&self, iteration: usize) -> Place {
        Place {
            iteration: Some(iteration),
            ..self.clone()
        }
    }

    pub fn called(&self, workflow: &str) -> Place {
        Place {
            path: format!("{}.{workflow}.steps", self.path),
            iteration: self.iteration,
            calls: self.calls + 1,
        }
    }
}
