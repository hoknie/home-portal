pub trait SecretSource: Send + Sync {
    fn reveal(&self, name: &str) -> Option<String>;

    fn is_set(&self, name: &str) -> bool {
        self.reveal(name).is_some()
    }
}
