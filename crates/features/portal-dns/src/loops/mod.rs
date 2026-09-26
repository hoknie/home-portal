mod binding;
mod datagrams;
mod secure;
mod streams;
mod supervisor;

#[cfg(test)]
mod tests;

pub use supervisor::DnsRuntime;
