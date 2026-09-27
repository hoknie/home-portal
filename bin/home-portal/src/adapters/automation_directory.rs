use portal_auth::UserNames;
use portal_automations::{Choice, Directory};
use portal_model::Environment;
use portal_network::CurrentEnvironments;
use portal_services::ServiceEntries;

pub struct AutomationDirectory {
    pub users: UserNames,
    pub services: ServiceEntries,
    pub environments: CurrentEnvironments,
}

impl Directory for AutomationDirectory {
    fn services(&self) -> Vec<Choice> {
        self.services
            .run()
            .map(|services| {
                services
                    .into_iter()
                    .map(|entry| Choice {
                        id: entry.id,
                        name: entry.name,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn users(&self) -> Vec<String> {
        self.users.run()
    }

    fn environments(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .environments
            .run()
            .map(|environments| {
                environments
                    .names()
                    .iter()
                    .map(|name| name.as_str().to_string())
                    .collect()
            })
            .unwrap_or_default();
        if !names.iter().any(|name| name == Environment::INTERNET) {
            names.push(Environment::INTERNET.to_string());
        }
        names
    }
}
