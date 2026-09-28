use super::{FieldDescription, KindGroup};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindDescription {
    pub name: &'static str,
    pub group: KindGroup,
    pub fields: &'static [FieldDescription],
    pub results: &'static [&'static str],
    pub exclusive: &'static [&'static [&'static str]],
}
