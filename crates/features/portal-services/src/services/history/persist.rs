use std::io;

use super::ServiceHistory;
use crate::repositories::HistoryFiles;
use crate::types::HistoryWrite;

pub fn persist(files: &HistoryFiles, id: &str, write: &HistoryWrite, now: i64) -> io::Result<()> {
    match write {
        HistoryWrite::Rewrite(history) => files.rewrite(id, &history.lines()),
        HistoryWrite::Append(lines) if lines.is_empty() => Ok(()),
        HistoryWrite::Append(lines) => {
            files.append(id, lines)?;
            if files.needs_compacting(id) {
                let history = ServiceHistory::from_lines(files.read(id)?, now);
                files.rewrite(id, &history.lines())?;
            }
            Ok(())
        }
    }
}
