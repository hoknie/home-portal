use std::process::Output;

#[derive(Debug)]
pub enum Ran {
    Finished(Output),
    TimedOut,
    NotStarted,
}
