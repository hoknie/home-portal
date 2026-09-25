use crate::types::Registry;

pub fn spawn(registry: &Registry) {
    for feature in &registry.features {
        for task in feature.loops() {
            tokio::spawn(task);
        }
    }
}
