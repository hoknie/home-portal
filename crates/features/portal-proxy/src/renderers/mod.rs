mod caddy;
mod routes;

#[cfg(test)]
mod tests;

pub use caddy::render;
pub use routes::{AUTHORIZE_PATH, USER_HEADER, portal_dial};
