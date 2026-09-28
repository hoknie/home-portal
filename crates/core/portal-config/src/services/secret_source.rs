use portal_feature::SecretSource;

use super::ConfigStore;

impl SecretSource for ConfigStore {
    fn reveal(&self, name: &str) -> Option<String> {
        self.secret(name).map(|secret| secret.expose().to_string())
    }
}
