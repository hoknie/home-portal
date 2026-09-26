mod serve;

#[cfg(test)]
mod tests;

pub use serve::{answer, interface, serve};
