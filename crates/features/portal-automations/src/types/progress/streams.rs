use crate::types::Tail;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Streams {
    pub stdout: Tail,
    pub stderr: Tail,
    pub command: Vec<String>,
    pub budget_reached: bool,
}

impl Streams {
    pub const KEPT_PER_STREAM: usize = 16 * 1024;
    pub const KEPT_PAST_BUDGET: usize = 1024;
    pub const MOST_BYTES_PER_RUN: usize = 128 * 1024;

    pub fn masked(self, mask: impl Fn(&str) -> String) -> Streams {
        let masked = |tail: &Tail| Tail::restored(mask(&tail.text()).as_bytes(), tail.total);
        Streams {
            stdout: masked(&self.stdout),
            stderr: masked(&self.stderr),
            command: self.command.iter().map(|part| mask(part)).collect(),
            budget_reached: self.budget_reached,
        }
    }

    pub fn kept(&self, most: usize) -> Streams {
        Streams {
            stdout: self.stdout.last(most),
            stderr: self.stderr.last(most),
            command: self.command.clone(),
            budget_reached: self.budget_reached,
        }
    }

    pub fn bytes(&self) -> usize {
        self.stdout.kept_bytes() + self.stderr.kept_bytes()
    }
}
