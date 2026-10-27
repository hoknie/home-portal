use std::fs;

use portal_model::{ProbeOutcome, ServiceState};

use crate::repositories::HistoryFiles;
use crate::services::ServiceHistory;
use crate::types::HistoryLine;

fn files() -> (tempfile::TempDir, HistoryFiles) {
    let directory = tempfile::tempdir().unwrap();
    let files = HistoryFiles::at(directory.path().join("history"));
    (directory, files)
}

fn lines() -> Vec<HistoryLine> {
    let mut history = ServiceHistory::default();
    history.record(1_790_000_000, &ProbeOutcome::answered(ServiceState::Up, 12));
    history.take_lines()
}

#[test]
fn appended_lines_are_read_back_in_order() {
    let (_directory, files) = files();
    files.append("media", &lines()).unwrap();
    files.append("media", &lines()).unwrap();
    assert_eq!(files.read("media").unwrap(), [lines(), lines()].concat());
}

#[cfg(unix)]
#[test]
fn a_history_file_and_its_folder_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let (directory, files) = files();
    files.append("media", &lines()).unwrap();
    let mode = |path: &std::path::Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&files.file_of("media")), 0o600);
    assert_eq!(mode(&directory.path().join("history")), 0o700);
}

#[test]
fn only_history_files_are_listed_and_leftover_temporary_files_are_removed() {
    let (directory, files) = files();
    files.append("media", &lines()).unwrap();
    let folder = directory.path().join("history");
    fs::write(folder.join("nas.ndjson.tmp"), "").unwrap();
    fs::write(folder.join("old.json"), "{}").unwrap();
    assert_eq!(files.ids(), ["media"]);
    assert!(!folder.join("nas.ndjson.tmp").exists());
    assert!(folder.join("old.json").exists());
}

#[test]
fn an_unreadable_file_is_set_aside_when_recovered() {
    let (directory, files) = files();
    fs::create_dir_all(directory.path().join("history")).unwrap();
    fs::write(files.file_of("media"), "this is not json").unwrap();
    assert_eq!(files.recover("media"), None);
    assert!(
        directory
            .path()
            .join("history/media.ndjson.broken")
            .is_file()
    );
    assert!(files.read("media").is_err());
}

#[test]
fn a_rewrite_replaces_the_file_and_resets_the_compaction_threshold() {
    let (_directory, files) = files();
    files.append("media", &lines()).unwrap();
    files.append("media", &lines()).unwrap();
    files.rewrite("media", &lines()).unwrap();
    assert_eq!(files.read("media").unwrap(), lines());
    assert!(!files.needs_compacting("media"));
    files.delete("media").unwrap();
    files.delete("media").unwrap();
    assert!(!files.file_of("media").exists());
}
