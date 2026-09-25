use std::collections::VecDeque;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tail {
    kept: VecDeque<u8>,
    pub total: u64,
}

impl Tail {
    pub const KEPT: usize = 4096;

    pub fn push(&mut self, chunk: &[u8]) {
        self.total += chunk.len() as u64;
        let start = chunk.len().saturating_sub(Self::KEPT);
        self.kept.extend(&chunk[start..]);
        let excess = self.kept.len().saturating_sub(Self::KEPT);
        self.kept.drain(..excess);
    }

    pub fn restored(kept: &[u8], total: u64) -> Tail {
        let start = kept.len().saturating_sub(Self::KEPT);
        Tail {
            kept: kept[start..].iter().copied().collect(),
            total: total.max(kept.len() as u64),
        }
    }

    pub fn text(&self) -> String {
        let bytes: Vec<u8> = self.kept.iter().copied().collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    pub fn truncated(&self) -> bool {
        self.total > self.kept.len() as u64
    }
}
