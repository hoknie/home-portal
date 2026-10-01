mod servers;
mod services;
mod types;

pub use servers::FakeHttp;
pub use services::{MAIN_FILE, opened, split, written};
pub use types::{Answer, Request};
