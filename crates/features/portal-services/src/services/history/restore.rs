use std::collections::HashMap;

use super::ServiceHistory;
use crate::repositories::HistoryFiles;

pub fn restore(
    files: &HistoryFiles,
    configured: &[String],
    now: i64,
) -> HashMap<String, ServiceHistory> {
    let mut restored = HashMap::new();
    for id in files.ids() {
        if !configured.iter().any(|known| known == &id) {
            if let Err(error) = files.delete(&id) {
                tracing::warn!(path = %files.file_of(&id).display(), %error, "cannot delete the history of a removed service");
            }
            continue;
        }
        let Some(lines) = files.recover(&id) else {
            continue;
        };
        let history = ServiceHistory::from_lines(lines, now);
        if let Err(error) = files.rewrite(&id, &history.lines()) {
            tracing::warn!(service = %id, %error, "cannot rewrite the service history");
        }
        restored.insert(id, history);
    }
    restored
}
