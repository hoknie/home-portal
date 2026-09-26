use std::collections::VecDeque;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tail {
    kept: VecDeque<u8>,
    pub total: u64,
}

impl Tail {
    pub const KEPT: usize = 64 * 1024;
    const CONTINUATION_MASK: u8 = 0b1100_0000;
    const CONTINUATION: u8 = 0b1000_0000;
    const LONGEST_CONTINUATION: usize = 3;

    pub fn push(&mut self, chunk: &[u8]) {
        self.total += chunk.len() as u64;
        let start = chunk.len().saturating_sub(Self::KEPT);
        self.kept.extend(&chunk[start..]);
        let excess = self.kept.len().saturating_sub(Self::KEPT);
        self.kept.drain(..excess);
        if self.truncated() {
            self.start_on_a_character();
        }
    }

    pub fn restored(kept: &[u8], total: u64) -> Tail {
        let start = kept.len().saturating_sub(Self::KEPT);
        let mut tail = Tail {
            kept: kept[start..].iter().copied().collect(),
            total: total.max(kept.len() as u64),
        };
        if tail.truncated() {
            tail.start_on_a_character();
        }
        tail
    }

    pub fn text(&self) -> String {
        let bytes: Vec<u8> = self.kept.iter().copied().collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    pub fn truncated(&self) -> bool {
        self.total > self.kept.len() as u64
    }

    fn start_on_a_character(&mut self) {
        let continuations = self
            .kept
            .iter()
            .take(Self::LONGEST_CONTINUATION)
            .take_while(|byte| *byte & Self::CONTINUATION_MASK == Self::CONTINUATION)
            .count();
        self.kept.drain(..continuations);
    }
}
