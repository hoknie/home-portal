use crate::usecases::{ChangeLayout, LibraryCases, ShowLayout};

#[derive(Clone)]
pub struct DashboardState {
    pub show: ShowLayout,
    pub change: ChangeLayout,
    pub library: LibraryCases,
}
