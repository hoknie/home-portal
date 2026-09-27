use crate::usecases::{
    ChangeService, CreateService, DeleteService, ListServices, ShowHistory, WakeProbe,
};

#[derive(Clone)]
pub struct ServicesState {
    pub list: ListServices,
    pub create: CreateService,
    pub change: ChangeService,
    pub delete: DeleteService,
    pub wake: WakeProbe,
    pub history: ShowHistory,
}
