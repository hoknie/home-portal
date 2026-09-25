use jiff::Timestamp;

use crate::ports::Clock;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now()
    }
}
