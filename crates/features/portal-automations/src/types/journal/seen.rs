use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    pub started_at: OffsetDateTime,
    pub last_at: OffsetDateTime,
    pub count: u32,
    pub absorbed: Vec<u64>,
}

impl Seen {
    pub const MOST_ABSORBED: usize = 1000;

    pub fn once(at: OffsetDateTime) -> Seen {
        Seen {
            started_at: at,
            last_at: at,
            count: 1,
            absorbed: Vec::new(),
        }
    }

    pub fn absorb(&mut self, id: u64, at: OffsetDateTime) {
        self.count += 1;
        self.last_at = at;
        self.absorbed.push(id);
        if self.absorbed.len() > Self::MOST_ABSORBED {
            self.absorbed.remove(0);
        }
    }

    pub fn holds(&self, id: u64) -> bool {
        self.absorbed.contains(&id)
    }
}
