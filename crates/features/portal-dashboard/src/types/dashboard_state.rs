use crate::usecases::{ChangeLayout, ShowLayout};

#[derive(Clone)]
pub struct DashboardState {
    pub show: ShowLayout,
    pub change: ChangeLayout,
}
