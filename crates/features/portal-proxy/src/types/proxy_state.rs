use std::sync::Arc;

use super::ProxyPorts;
use crate::loops::CaddySync;
use crate::usecases::{
    ApplyProxy, ChangeCaddySource, ChangeProxy, DownloadCaddy, ShowProxy, StartCaddy, StopCaddy,
};

#[derive(Clone)]
pub struct ProxyState {
    pub show: ShowProxy,
    pub apply: ApplyProxy,
    pub change: ChangeProxy,
    pub download: DownloadCaddy,
    pub start: StartCaddy,
    pub stop: StopCaddy,
    pub change_source: ChangeCaddySource,
    pub ports: ProxyPorts,
    pub sync: Arc<CaddySync>,
}
