#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyCode {
    NoError,
    FormatError,
    NameError,
    NotImplemented,
    Refused,
}
