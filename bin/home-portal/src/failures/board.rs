use std::sync::{Arc, Mutex, PoisonError};

use time::OffsetDateTime;

use super::main_file::MainFile;
use super::problems::Problem;

#[derive(Debug, Clone)]
pub struct Failure {
    pub since: OffsetDateTime,
    pub checked: OffsetDateTime,
    pub problems: Vec<Problem>,
    pub main: MainFile,
}

#[derive(Debug, Clone)]
pub struct FailureBoard {
    current: Arc<Mutex<Failure>>,
}

impl FailureBoard {
    pub fn new(problems: Vec<Problem>, main: MainFile) -> FailureBoard {
        let now = OffsetDateTime::now_utc();
        FailureBoard {
            current: Arc::new(Mutex::new(Failure {
                since: now,
                checked: now,
                problems,
                main,
            })),
        }
    }

    pub fn current(&self) -> Failure {
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn checked(&self, problems: Vec<Problem>, main: MainFile) -> bool {
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        let changed = current.problems != problems;
        current.checked = OffsetDateTime::now_utc();
        current.problems = problems;
        current.main = main;
        changed
    }
}
