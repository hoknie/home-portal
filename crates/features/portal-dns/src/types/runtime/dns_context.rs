use crate::usecases::{ChangeDns, ShowDns};

#[derive(Clone)]
pub struct DnsContext {
    pub show: ShowDns,
    pub change: ChangeDns,
}
