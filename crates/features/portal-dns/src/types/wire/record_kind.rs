#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RecordKind {
    A,
    Aaaa,
    Cname,
    Txt,
    Soa,
    Ns,
    Any,
    Transfer,
    Other,
}

impl RecordKind {
    pub const WRITABLE: [RecordKind; 4] = [
        RecordKind::A,
        RecordKind::Aaaa,
        RecordKind::Cname,
        RecordKind::Txt,
    ];

    pub fn parse(text: &str) -> Option<RecordKind> {
        match text {
            "A" => Some(RecordKind::A),
            "AAAA" => Some(RecordKind::Aaaa),
            "CNAME" => Some(RecordKind::Cname),
            "TXT" => Some(RecordKind::Txt),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            RecordKind::A => "A",
            RecordKind::Aaaa => "AAAA",
            RecordKind::Cname => "CNAME",
            RecordKind::Txt => "TXT",
            RecordKind::Soa => "SOA",
            RecordKind::Ns => "NS",
            RecordKind::Any => "ANY",
            RecordKind::Transfer => "AXFR",
            RecordKind::Other => "OTHER",
        }
    }
}
