mod destinations;
mod loopback;

#[cfg(test)]
mod tests;

pub use destinations::{
    FORWARDED_METHOD, FORWARDED_URI, continuation, forwarded, forwarded_host, sign_in_address,
};
pub use loopback::{LOOPBACK, covers_loopback};
