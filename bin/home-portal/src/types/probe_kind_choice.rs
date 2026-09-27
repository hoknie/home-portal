use clap::ValueEnum;
use portal_services::ProbeKind;

pub const HTTP_HELP: &str = "an HTTP request; the status code decides the state";
pub const TCP_HELP: &str = "a connection to the port, nothing sent";
pub const ICMP_HELP: &str = "one echo request";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProbeKindChoice {
    #[value(help = HTTP_HELP)]
    Http,
    #[value(help = TCP_HELP)]
    Tcp,
    #[value(help = ICMP_HELP)]
    Icmp,
}

impl From<ProbeKindChoice> for ProbeKind {
    fn from(choice: ProbeKindChoice) -> ProbeKind {
        match choice {
            ProbeKindChoice::Http => ProbeKind::Http,
            ProbeKindChoice::Tcp => ProbeKind::Tcp,
            ProbeKindChoice::Icmp => ProbeKind::Icmp,
        }
    }
}
