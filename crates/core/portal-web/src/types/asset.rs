use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Asset {
    pub bytes: Cow<'static, [u8]>,
    pub content_type: String,
}
