pub struct PlacementKey;

impl PlacementKey {
    pub const PREFIX: &'static str = "#";

    pub fn of(index: usize) -> String {
        format!("{}{index}", Self::PREFIX)
    }
}
