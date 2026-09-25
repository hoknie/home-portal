mod advice;
mod backoff;
mod echo;
mod failure;
mod local_network;
mod target;

#[cfg(test)]
mod tests;

pub use advice::advice;
pub use backoff::next_wait;
pub use echo::{TOKEN_LENGTH, echo_request, is_echo_reply};
pub use failure::{classify_failure, classify_io};
pub use local_network::{PLATFORM_REFUSES_LOCAL_NETWORK, local_network_denied, refine_unreachable};
pub use target::{probe_host, probe_target, tcp_port};
