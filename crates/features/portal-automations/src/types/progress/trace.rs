use std::time::Duration;

use time::OffsetDateTime;

use super::{StepOutcome, TraceEntry};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Trace {
    pub entries: Vec<TraceEntry>,
    pub dropped: usize,
}

impl Trace {
    pub const MOST_ENTRIES: usize = 200;

    pub fn start(&mut self, entry: TraceEntry) -> Option<usize> {
        if self.entries.len() >= Self::MOST_ENTRIES {
            self.dropped += 1;
            return None;
        }
        self.entries.push(entry);
        Some(self.entries.len() - 1)
    }

    pub fn finish(
        &mut self,
        index: Option<usize>,
        (outcome, detail, output, shape): (StepOutcome, String, Option<String>, Option<String>),
        now: OffsetDateTime,
    ) {
        let Some(entry) = index.and_then(|index| self.entries.get_mut(index)) else {
            return;
        };
        entry.outcome = outcome;
        entry.detail = cut(&detail, TraceEntry::LONGEST_DETAIL);
        entry.output = output.map(|output| cut(&output, TraceEntry::LONGEST_OUTPUT));
        entry.shape = shape;
        entry.duration = Duration::try_from(now - entry.started_at).unwrap_or_default();
    }
}

fn cut(text: &str, longest: usize) -> String {
    if text.len() <= longest {
        return text.to_string();
    }
    let mut end = longest;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}
