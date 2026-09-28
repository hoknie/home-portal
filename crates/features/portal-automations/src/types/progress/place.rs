#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Place {
    pub path: String,
    pub iteration: Option<usize>,
    pub calls: usize,
    pub item: Option<String>,
}

impl Place {
    pub fn root() -> Place {
        Place {
            path: "steps".to_string(),
            iteration: None,
            calls: 0,
            item: None,
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
            item: None,
            ..self.clone()
        }
    }

    pub fn with_item(self, item: Option<String>) -> Place {
        Place { item, ..self }
    }

    pub fn called(&self, workflow: &str) -> Place {
        Place {
            path: format!("{}.{workflow}.steps", self.path),
            iteration: self.iteration,
            calls: self.calls + 1,
            item: self.item.clone(),
        }
    }
}
