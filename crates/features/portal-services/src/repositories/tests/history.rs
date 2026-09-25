use std::fs;
use std::sync::Arc;

use portal_model::{ProbeOutcome, ServiceState};
use time::OffsetDateTime;

use crate::loops::HistoryWriter;
use crate::repositories::HistoryFiles;
use crate::services::{ServiceHistory, StatusBoard};
use crate::types::{HistoryWrite, ServiceEntry};

fn files() -> (tempfile::TempDir, HistoryFiles) {
    let directory = tempfile::tempdir().unwrap();
    let files = HistoryFiles::at(directory.path().join("history"));
    (directory, files)
}

fn appended(history: &ServiceHistory) -> HistoryWrite {
    HistoryWrite::Append(history.clone().take_lines())
}

fn history() -> ServiceHistory {
    let now = OffsetDateTime::now_utc().unix_timestamp() - 60;
    let mut history = ServiceHistory::default();
    history.record(now, &ProbeOutcome::answered(ServiceState::Up, 12));
    history.record(
        now + 30,
        &ProbeOutcome::failed(ServiceState::Down, None, "refused".into()),
    );
    history
}

#[test]
fn a_saved_history_is_loaded_back_beside_the_configuration() {
    let (directory, files) = files();
    files.save("media", &appended(&history())).unwrap();
    assert!(directory.path().join("history/media.ndjson").is_file());
    let loaded = files.load(&["media".to_string()]);
    assert_eq!(loaded["media"].samples, history().samples);
    assert_eq!(loaded["media"].transitions, history().transitions);
}

#[cfg(unix)]
#[test]
fn a_history_file_is_readable_only_by_its_owner() {
    use std::os::unix::fs::PermissionsExt;
    let (_directory, files) = files();
    files.save("media", &appended(&history())).unwrap();
    let mode = fs::metadata(files.file_of("media"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn a_garbage_file_is_set_aside_and_the_service_starts_empty() {
    let (directory, files) = files();
    fs::create_dir_all(directory.path().join("history")).unwrap();
    fs::write(files.file_of("media"), "this is not json").unwrap();
    let loaded = files.load(&["media".to_string()]);
    assert!(loaded.is_empty());
    assert!(!files.file_of("media").exists());
    assert!(
        directory
            .path()
            .join("history/media.ndjson.broken")
            .is_file()
    );
}

#[test]
fn the_file_of_a_service_no_longer_configured_is_deleted_at_start() {
    let (_directory, files) = files();
    files.save("gone", &appended(&history())).unwrap();
    let loaded = files.load(&["media".to_string()]);
    assert!(loaded.is_empty());
    assert!(!files.file_of("gone").exists());
}

#[test]
fn each_save_appends_only_the_new_lines_one_json_object_per_line() {
    let (_directory, files) = files();
    let mut history = history();
    files
        .save("media", &HistoryWrite::Append(history.take_lines()))
        .unwrap();
    let first = fs::read_to_string(files.file_of("media")).unwrap();
    assert!(
        first
            .lines()
            .all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok())
    );
    let later = OffsetDateTime::now_utc().unix_timestamp();
    history.record(later, &ProbeOutcome::answered(ServiceState::Up, 9));
    files
        .save("media", &HistoryWrite::Append(history.take_lines()))
        .unwrap();
    let second = fs::read_to_string(files.file_of("media")).unwrap();
    assert!(second.starts_with(&first));
    assert!(second.lines().count() > first.lines().count());
    let loaded = files.load(&["media".to_string()]);
    assert_eq!(loaded["media"].samples.back().unwrap().at, later);
}

#[test]
fn a_line_cut_off_by_a_crash_is_skipped_and_the_rest_is_kept() {
    let (_directory, files) = files();
    files.save("media", &appended(&history())).unwrap();
    let mut text = fs::read_to_string(files.file_of("media")).unwrap();
    text.push_str("{\"kind\":\"sample\",\"at\":");
    fs::write(files.file_of("media"), text).unwrap();
    let loaded = files.load(&["media".to_string()]);
    assert_eq!(loaded["media"].samples, history().samples);
    let rewritten = fs::read_to_string(files.file_of("media")).unwrap();
    assert!(
        rewritten
            .lines()
            .all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok())
    );
}

#[test]
fn a_history_in_the_former_json_format_is_carried_over_to_ndjson() {
    let (directory, files) = files();
    fs::create_dir_all(directory.path().join("history")).unwrap();
    let legacy = directory.path().join("history/media.json");
    fs::write(&legacy, serde_json::to_string(&history()).unwrap()).unwrap();
    let loaded = files.load(&["media".to_string()]);
    assert_eq!(loaded["media"].transitions, history().transitions);
    assert!(!legacy.exists());
    assert!(files.file_of("media").is_file());
}

#[test]
fn a_flush_writes_dirty_histories_and_deletes_forgotten_ones() {
    let (_directory, files) = files();
    let files = Arc::new(files);
    let board = Arc::new(StatusBoard::watched(OffsetDateTime::now_utc(), Vec::new()));
    let writer = HistoryWriter::new(board.clone(), files.clone());
    let media = ServiceEntry::new("media", "Media", "http://10.0.0.5");
    board.record(
        &media,
        ProbeOutcome::answered(ServiceState::Up, 5),
        OffsetDateTime::now_utc(),
    );
    writer.flush();
    assert!(files.file_of("media").is_file());
    let modified = fs::metadata(files.file_of("media"))
        .unwrap()
        .modified()
        .unwrap();
    writer.flush();
    assert_eq!(
        fs::metadata(files.file_of("media"))
            .unwrap()
            .modified()
            .unwrap(),
        modified
    );
    board.forget("media");
    writer.flush();
    assert!(!files.file_of("media").exists());
}

fn board_and_writer() -> (
    tempfile::TempDir,
    Arc<StatusBoard>,
    HistoryWriter,
    Arc<HistoryFiles>,
) {
    let (directory, files) = files();
    let files = Arc::new(files);
    let board = Arc::new(StatusBoard::watched(OffsetDateTime::now_utc(), Vec::new()));
    let writer = HistoryWriter::new(board.clone(), files.clone());
    (directory, board, writer, files)
}

#[test]
fn a_renamed_service_keeps_its_whole_history() {
    let (_directory, board, writer, files) = board_and_writer();
    let media = ServiceEntry::new("media", "Media", "http://10.0.0.5");
    board.record(
        &media,
        ProbeOutcome::answered(ServiceState::Up, 5),
        OffsetDateTime::now_utc(),
    );
    writer.flush();
    board.rename("media", "films");
    writer.flush();
    assert!(!files.file_of("media").exists());
    let loaded = files.load(&["films".to_string()]);
    assert_eq!(loaded["films"].samples.len(), 1);
}

#[test]
fn a_service_forgotten_by_a_reconcile_just_before_its_rename_keeps_its_history() {
    let (_directory, board, writer, files) = board_and_writer();
    let media = ServiceEntry::new("media", "Media", "http://10.0.0.5");
    board.record(
        &media,
        ProbeOutcome::answered(ServiceState::Up, 5),
        OffsetDateTime::now_utc(),
    );
    writer.flush();
    board.forget("media");
    board.rename("media", "films");
    writer.flush();
    assert!(!files.file_of("media").exists());
    let loaded = files.load(&["films".to_string()]);
    assert_eq!(loaded["films"].samples.len(), 1);
}

#[test]
fn a_service_created_again_replaces_the_file_of_the_deleted_one() {
    let (_directory, board, writer, files) = board_and_writer();
    let media = ServiceEntry::new("media", "Media", "http://10.0.0.5");
    let past = OffsetDateTime::now_utc() - time::Duration::minutes(10);
    board.record(
        &media,
        ProbeOutcome::failed(ServiceState::Down, None, "old".into()),
        past,
    );
    writer.flush();
    board.forget("media");
    board.record(
        &media,
        ProbeOutcome::answered(ServiceState::Up, 5),
        OffsetDateTime::now_utc(),
    );
    writer.flush();
    let loaded = files.load(&["media".to_string()]);
    assert_eq!(loaded["media"].samples.len(), 1);
    assert_eq!(loaded["media"].samples[0].state, ServiceState::Up);
}

#[cfg(unix)]
#[test]
fn a_failed_write_rewrites_the_whole_history_next_time() {
    use std::os::unix::fs::PermissionsExt;
    let (directory, board, writer, files) = board_and_writer();
    let media = ServiceEntry::new("media", "Media", "http://10.0.0.5");
    board.record(
        &media,
        ProbeOutcome::answered(ServiceState::Up, 5),
        OffsetDateTime::now_utc(),
    );
    writer.flush();
    let history = directory.path().join("history");
    fs::set_permissions(&history, fs::Permissions::from_mode(0o500)).unwrap();
    let later = OffsetDateTime::now_utc() + time::Duration::seconds(30);
    board.record(
        &media,
        ProbeOutcome::failed(ServiceState::Down, None, "refused".into()),
        later,
    );
    writer.flush();
    fs::set_permissions(&history, fs::Permissions::from_mode(0o700)).unwrap();
    writer.flush();
    let loaded = files.load(&["media".to_string()]);
    assert_eq!(loaded["media"].samples.len(), 2);
    assert_eq!(loaded["media"].transitions.len(), 2);
}

#[test]
fn a_line_with_broken_bytes_is_skipped_and_later_lines_are_kept() {
    let (_directory, files) = files();
    let mut history = history();
    files
        .save("media", &HistoryWrite::Append(history.take_lines()))
        .unwrap();
    let path = files.file_of("media");
    let mut bytes = fs::read(&path).unwrap();
    bytes.extend_from_slice(b"{\"kind\":\"transition\",\"error\":\"\xd0\n");
    fs::write(&path, bytes).unwrap();
    let later = OffsetDateTime::now_utc().unix_timestamp();
    history.record(later, &ProbeOutcome::answered(ServiceState::Up, 3));
    files
        .save("media", &HistoryWrite::Append(history.take_lines()))
        .unwrap();
    let loaded = files.load(&["media".to_string()]);
    assert_eq!(loaded["media"].samples.back().unwrap().at, later);
}

#[test]
fn a_former_json_and_newer_lines_are_merged_and_the_json_goes_away() {
    let (directory, files) = files();
    fs::create_dir_all(directory.path().join("history")).unwrap();
    let legacy = directory.path().join("history/media.json");
    fs::write(&legacy, serde_json::to_string(&history()).unwrap()).unwrap();
    let mut newer = ServiceHistory::default();
    let later = OffsetDateTime::now_utc().unix_timestamp();
    newer.record(later, &ProbeOutcome::answered(ServiceState::Up, 4));
    files
        .save("media", &HistoryWrite::Append(newer.take_lines()))
        .unwrap();
    let loaded = files.load(&["media".to_string()]);
    assert_eq!(loaded["media"].samples.len(), 3);
    assert!(!legacy.exists());
}
