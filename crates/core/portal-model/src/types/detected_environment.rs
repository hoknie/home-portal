use super::{Environment, Environments};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedEnvironment {
    pub environment: Environment,
    pub choices: Vec<Environment>,
}

impl DetectedEnvironment {
    pub fn new(environment: Environment, environments: &Environments) -> DetectedEnvironment {
        let choices = if environment.is_internet() {
            Vec::new()
        } else {
            environments.names()
        };
        DetectedEnvironment {
            environment,
            choices,
        }
    }

    pub fn switchable(&self) -> bool {
        !self.choices.is_empty()
    }
}
