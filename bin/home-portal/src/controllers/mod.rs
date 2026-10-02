mod failure;
mod not_found;
mod restart;

pub use failure::{
    FAILURE_PATH, FATAL_PAGE, FATAL_PATH, failed_health, failure_report, fatal_to_home,
    not_started, page_or_fatal,
};
pub use not_found::not_found;
pub use restart::{RESTART_PATH, restart};
