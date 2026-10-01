mod splitting;
mod writing;

#[cfg(test)]
mod tests;

pub use splitting::{opened, split};
pub use writing::{MAIN_FILE, written};
