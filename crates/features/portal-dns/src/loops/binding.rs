use tokio::task::JoinHandle;

pub struct Binding<K> {
    pub key: K,
    pub tasks: Vec<JoinHandle<()>>,
}

impl<K> Drop for Binding<K> {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}
