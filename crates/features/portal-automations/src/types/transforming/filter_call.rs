use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterCall {
    pub name: String,
    pub arguments: Vec<Value>,
    pub names: Vec<(usize, String)>,
}

impl FilterCall {
    pub fn literal(name: &str, arguments: Vec<Value>) -> FilterCall {
        FilterCall {
            name: name.to_string(),
            arguments,
            names: Vec::new(),
        }
    }

    pub fn is_named(&self, position: usize) -> bool {
        self.names.iter().any(|(at, _)| *at == position)
    }
}
