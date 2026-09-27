use super::EffectiveAddress;
use crate::usecases::{ChangeNetwork, CurrentEnvironments, ShowNetwork};

#[derive(Clone)]
pub struct NetworkState {
    pub show: ShowNetwork,
    pub change: ChangeNetwork,
    pub environments: CurrentEnvironments,
    pub effective: EffectiveAddress,
}
