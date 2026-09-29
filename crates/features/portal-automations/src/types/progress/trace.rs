use std::time::Duration;

use time::OffsetDateTime;

use super::{Streams, TraceEntry};
use crate::types::{EntryEnd, StepLog};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Trace {
    pub entries: Vec<TraceEntry>,
    pub dropped: usize,
    pub log_bytes: usize,
    pub output_bytes: usize,
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

    pub fn finish(&mut self, index: Option<usize>, end: EntryEnd, now: OffsetDateTime) {
        let Some(index) = index.filter(|index| *index < self.entries.len()) else {
            return;
        };
        let streams = end.streams.map(|streams| self.budgeted(streams));
        let entry = &mut self.entries[index];
        let bytes = end.log.bytes();
        entry.log = if self.log_bytes + bytes > StepLog::MOST_BYTES_PER_RUN {
            end.log.dropped()
        } else {
            self.log_bytes += bytes;
            end.log
        };
        entry.outcome = end.outcome;
        entry.detail = cut(&end.detail, TraceEntry::LONGEST_DETAIL);
        entry.output = end
            .output
            .map(|output| cut(&output, TraceEntry::LONGEST_OUTPUT));
        entry.shape = end.shape;
        entry.streams = streams;
        entry.duration = Duration::try_from(now - entry.started_at).unwrap_or_default();
    }

    fn budgeted(&mut self, streams: Streams) -> Streams {
        let kept = streams.kept(Streams::KEPT_PER_STREAM);
        if self.output_bytes + kept.bytes() <= Streams::MOST_BYTES_PER_RUN {
            self.output_bytes += kept.bytes();
            return kept;
        }
        Streams {
            budget_reached: true,
            ..streams.kept(Streams::KEPT_PAST_BUDGET)
        }
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
