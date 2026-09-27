mod dns_choice;
mod dns_https_choice;
mod dns_record;
mod dns_settings;
mod dns_tls_choice;
mod raw_dns;

pub use dns_choice::DnsChoice;
pub use dns_https_choice::DnsHttpsChoice;
pub use dns_record::DnsRecord;
pub use dns_settings::{DnsHttps, DnsSettings, DnsTls, ProxyAddresses, ProxyView};
pub use dns_tls_choice::DnsTlsChoice;
pub use raw_dns::{RawAddresses, RawDns, RawDnsSection, RawRecord};
