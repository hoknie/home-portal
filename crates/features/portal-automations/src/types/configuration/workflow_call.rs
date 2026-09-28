use crate::types::InputValue;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowCall {
    pub id: String,
    pub inputs: Vec<(String, InputValue)>,
}

impl WorkflowCall {
    pub fn templates(&self) -> impl Iterator<Item = (String, &str)> {
        self.inputs.iter().filter_map(|(name, value)| {
            value
                .template()
                .map(|template| (format!("inputs.{name}"), template))
        })
    }
}
