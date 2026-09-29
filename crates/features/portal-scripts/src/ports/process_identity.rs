pub trait ProcessIdentity: Send + Sync {
    fn user(&self) -> u32;
    fn groups(&self) -> Vec<u32>;
}
