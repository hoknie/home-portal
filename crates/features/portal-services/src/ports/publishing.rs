pub trait Publishing: Send + Sync {
    fn https_port(&self) -> Option<u16>;
}
