mod proxy_section;

#[cfg(test)]
mod tests;

pub use proxy_section::{
    NETWORK, SECTION, append_trusted_proxy, trusted_proxies, write_caddy_source, write_choice,
    write_managed,
};
