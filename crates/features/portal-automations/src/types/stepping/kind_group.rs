#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindGroup {
    Flow,
    Data,
    Actions,
}

impl KindGroup {
    pub fn name(self) -> &'static str {
        match self {
            KindGroup::Flow => "flow",
            KindGroup::Data => "data",
            KindGroup::Actions => "actions",
        }
    }
}
