mod proxy_section;

#[cfg(test)]
mod tests;

pub use proxy_section::{
    NETWORK, SECTION, trust_loopback, write_caddy_source, write_choice, write_managed,
};
