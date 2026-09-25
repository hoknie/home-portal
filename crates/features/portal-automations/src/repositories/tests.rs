use toml_edit::DocumentMut;

use super::{append, position, remove, replace};
use crate::types::{Automation, AutomationsSection};

const FILE: &str = r#"# The portal's automations

# Restarts the media server when it falls over.
[[automations]]
id = "restart-media"
title = "Restart"
cooldown_seconds = 300 # five minutes
when = { event = "service.status-changed", services = ["jellyfin"], to = ["down"] }
run = { script = "restart.sh", args = ["--", "{{service.id}}"] }

# Nightly.
[[automations]]
id = "backup"
title = "Backup"
when = { event = "schedule", cron = "0 3 * * *" }
run = { script = "backup.sh", timeout_seconds = 600 }
"#;

fn decoded(document: &DocumentMut, index: usize) -> Automation {
    let section = AutomationsSection::read(document).unwrap();
    Automation::decode(&section.automations[index]).unwrap()
}

#[test]
fn a_title_change_keeps_comments_and_untouched_keys_byte_for_byte() {
    let mut document: DocumentMut = FILE.parse().unwrap();
    let mut automation = decoded(&document, 0);
    automation.title = "Restart Jellyfin".into();
    replace(&mut document, 0, &automation);
    let expected = FILE.replace("title = \"Restart\"", "title = \"Restart Jellyfin\"");
    assert_eq!(document.to_string(), expected);
}

#[test]
fn an_appended_automation_reads_back_as_it_was_written() {
    let mut document: DocumentMut = FILE.parse().unwrap();
    let mut automation = decoded(&document, 1);
    automation.id = "weekly".into();
    automation.enabled = false;
    automation.trigger.filters.cron.as_mut().unwrap().expression = "0 4 * * sun".into();
    append(&mut document, &automation);
    let text = document.to_string();
    assert!(text.ends_with(
        "[[automations]]\nid = \"weekly\"\ntitle = \"Backup\"\nenabled = false\nwhen = { event = \"schedule\", cron = \"0 4 * * sun\" }\nrun = { script = \"backup.sh\", timeout_seconds = 600 }\n"
    ), "{text}");
    assert_eq!(position(&document, "weekly"), Some(2));
}

#[test]
fn removing_the_first_keeps_the_comment_of_the_file_and_of_the_next() {
    let mut document: DocumentMut = FILE.parse().unwrap();
    remove(&mut document, 0);
    let text = document.to_string();
    assert!(
        text.starts_with(
            "# The portal's automations\n\n# Nightly.\n[[automations]]\nid = \"backup\""
        ),
        "{text}"
    );
    assert_eq!(position(&document, "restart-media"), None);
}

#[test]
fn a_run_written_as_a_sub_table_is_edited_in_place_with_its_comments() {
    let text = r#"[[automations]]
id = "backup"
title = "Backup"
when = { event = "schedule", cron = "0 3 * * *" }

[automations.run]
# the nightly script
script = "backup.sh" # keep it small
timeout_seconds = 600
"#;
    let mut document: DocumentMut = text.parse().unwrap();
    let mut automation = decoded(&document, 0);
    automation.title = "Nightly backup".into();
    automation.run.timeout_seconds = 900;
    replace(&mut document, 0, &automation);
    let expected = text
        .replace("title = \"Backup\"", "title = \"Nightly backup\"")
        .replace("timeout_seconds = 600", "timeout_seconds = 900");
    assert_eq!(document.to_string(), expected);
}

#[test]
fn a_webhook_edit_keeps_its_token_and_comments() {
    use super::{append_webhook, replace_webhook, set_token};
    use crate::types::Webhook;

    let text = "# hooks\n[[webhooks]]\nid = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"Deploy\" # from CI\naction = \"event\"\ntoken_sha256 = \"0000000000000000000000000000000000000000000000000000000000000000\"\n";
    let mut document: DocumentMut = text.parse().unwrap();
    let raw = AutomationsSection::read(&document)
        .unwrap()
        .webhooks
        .remove(0);
    let mut webhook = Webhook::decode(&raw).unwrap();
    webhook.variables = vec!["branch".into()];
    replace_webhook(&mut document, 0, &webhook);
    let written = document.to_string();
    assert!(
        written.contains("title = \"Deploy\" # from CI"),
        "{written}"
    );
    assert!(written.contains("token_sha256 = \"0000"), "{written}");
    assert!(written.contains("variables = [\"branch\"]"), "{written}");
    set_token(&mut document, 0, None);
    assert!(!document.to_string().contains("token_sha256"));
    webhook.id = "0b9e8c2a-1d3f-4a5b-8c7d-9e0f1a2b3c4d".into();
    append_webhook(&mut document, &webhook);
    assert_eq!(
        AutomationsSection::read(&document).unwrap().webhooks.len(),
        2
    );
}

mod run_file {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use time::{Duration, OffsetDateTime};

    use crate::repositories::RunFile;
    use crate::services::Journal;
    use crate::services::tests::support::pending;
    use crate::types::{RunRecord, SkipReason};

    fn at(seconds: i64) -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH + Duration::seconds(seconds)
    }

    fn file() -> (tempfile::TempDir, RunFile) {
        let folder = tempfile::tempdir().unwrap();
        let file = RunFile::at(&folder.path().join("automations"));
        (folder, file)
    }

    #[test]
    fn the_journal_survives_a_restart_with_merged_skips_and_its_order() {
        let (folder, file) = file();
        let journal = Journal::default();
        journal.record(RunRecord::skipped(
            &pending("a", 1, None),
            SkipReason::Cooldown,
            at(1),
        ));
        journal.record(RunRecord::skipped(
            &pending("b", 2, None),
            SkipReason::Running,
            at(2),
        ));
        journal.record(RunRecord::skipped(
            &pending("a", 3, None),
            SkipReason::Cooldown,
            at(3),
        ));
        file.append(&journal.take_unwritten()).unwrap();
        let text = fs::read_to_string(folder.path().join("automations/runs.ndjson")).unwrap();
        assert_eq!(text.lines().count(), 3);
        let (restored, _) = file.load(Journal::KEPT);
        assert_eq!(
            restored.iter().map(|run| run.id).collect::<Vec<_>>(),
            vec![2, 1]
        );
        assert_eq!(restored[1].seen.count, 2);
        assert_eq!(restored[1].seen.last_at, at(3));
        let mode = fs::metadata(folder.path().join("automations/runs.ndjson"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn a_line_cut_short_is_skipped() {
        let (folder, file) = file();
        file.append(&[RunRecord::skipped(
            &pending("a", 1, None),
            SkipReason::Cooldown,
            at(1),
        )])
        .unwrap();
        let path = folder.path().join("automations/runs.ndjson");
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("{\"id\":2,\"automation\":");
        fs::write(&path, text).unwrap();
        let (loaded, highest) = file.load(Journal::KEPT);
        assert_eq!(loaded.len(), 1);
        assert_eq!(highest, 2);
    }

    #[test]
    fn compacting_keeps_the_newest_runs_only() {
        let (_folder, file) = file();
        let records: Vec<RunRecord> = (1..=250)
            .map(|id| {
                RunRecord::skipped(
                    &pending(&format!("a{id}"), id, None),
                    SkipReason::Running,
                    at(id as i64),
                )
            })
            .collect();
        file.append(&records).unwrap();
        let (newest_first, _) = file.load(Journal::KEPT);
        assert_eq!(newest_first.len(), 200);
        assert_eq!(newest_first[0].id, 250);
        file.rewrite(&newest_first).unwrap();
        assert_eq!(file.load(Journal::KEPT).0[199].id, 51);
    }

    #[test]
    fn a_line_with_broken_bytes_is_skipped_and_its_id_is_not_reused() {
        let (folder, file) = file();
        let path = folder.path().join("automations/runs.ndjson");
        file.append(&[RunRecord::skipped(
            &pending("a", 1, None),
            SkipReason::Cooldown,
            at(1),
        )])
        .unwrap();
        let mut bytes = fs::read(&path).unwrap();
        bytes.extend_from_slice(b"{\"id\":7,\"automation\":\"\xff\xfe\"}\n");
        fs::write(&path, &bytes).unwrap();
        file.append(&[RunRecord::skipped(
            &pending("b", 3, None),
            SkipReason::Cooldown,
            at(3),
        )])
        .unwrap();
        let (loaded, highest) = file.load(Journal::KEPT);
        assert_eq!(
            loaded.iter().map(|run| run.id).collect::<Vec<_>>(),
            vec![3, 1]
        );
        assert_eq!(highest, 7);
    }

    #[test]
    fn a_leftover_temporary_file_is_removed_on_load() {
        let (folder, file) = file();
        fs::create_dir(folder.path().join("automations")).unwrap();
        let temporary = folder.path().join("automations/runs.ndjson.tmp");
        fs::write(&temporary, "half").unwrap();
        file.load(Journal::KEPT);
        assert!(!temporary.exists());
    }

    #[test]
    fn a_failed_write_is_written_whole_by_the_next_flush() {
        use std::sync::Arc;

        use crate::loops::JournalWriter;

        let (folder, file) = file();
        let blocker = folder.path().join("automations");
        fs::write(&blocker, "not a directory").unwrap();
        let journal = Arc::new(Journal::default());
        let writer = JournalWriter::new(journal.clone(), Arc::new(file));
        journal.record(RunRecord::skipped(
            &pending("a", 1, None),
            SkipReason::Cooldown,
            at(1),
        ));
        writer.flush();
        fs::remove_file(&blocker).unwrap();
        journal.record(RunRecord::skipped(
            &pending("b", 2, None),
            SkipReason::Cooldown,
            at(2),
        ));
        writer.flush();
        let (loaded, _) = writer.file.load(Journal::KEPT);
        assert_eq!(
            loaded.iter().map(|run| run.id).collect::<Vec<_>>(),
            vec![2, 1]
        );
    }

    #[test]
    fn compacting_waits_for_twice_the_size_left_by_the_last_compaction() {
        let (folder, file) = file();
        let output = vec![b'x'; 4096];
        let records: Vec<RunRecord> = (1..=200)
            .map(|id| {
                let mut run =
                    RunRecord::skipped(&pending("a", id, None), SkipReason::Running, at(id as i64));
                run.result.stdout.push(&output);
                run
            })
            .collect();
        let newest_first: Vec<RunRecord> = records.iter().rev().cloned().collect();
        file.rewrite(&newest_first).unwrap();
        let size = fs::metadata(folder.path().join("automations/runs.ndjson"))
            .unwrap()
            .len();
        assert!(size > RunFile::COMPACT_AFTER_BYTES / 2);
        file.append(&records[..100]).unwrap();
        assert!(!file.needs_compacting());
        file.append(&records).unwrap();
        file.append(&records).unwrap();
        assert!(file.needs_compacting());
    }
}
