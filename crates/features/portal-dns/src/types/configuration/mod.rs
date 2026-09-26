mod dns_record;
mod dns_settings;
mod raw_dns;

pub use dns_record::DnsRecord;
pub use dns_settings::{DnsHttps, DnsSettings, DnsTls, ProxyAddresses, ProxyView};
pub use raw_dns::{RawAddresses, RawDns, RawDnsSection, RawRecord};
