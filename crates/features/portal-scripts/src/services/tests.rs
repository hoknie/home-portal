use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;

use tempfile::TempDir;

use std::sync::Arc;

use crate::fakes::FileOwner;
use crate::services::ScriptsDirectory;

fn directory() -> (TempDir, ScriptsDirectory) {
    let folder = TempDir::new().unwrap();
    let root = folder.path().join("scripts");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    let directory = ScriptsDirectory::at(
        folder.path().join("scripts"),
        Arc::new(FileOwner::of_this_process()),
    );
    (folder, directory)
}

fn file(root: &Path, name: &str, mode: u32) {
    let path = root.join(name);
    fs::write(&path, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
}

fn refusal(directory: &ScriptsDirectory, name: &str) -> String {
    directory.resolve(name).unwrap_err().message
}

#[test]
fn a_private_executable_inside_the_directory_is_accepted() {
    let (_folder, directory) = directory();
    file(directory.root(), "backup.sh", 0o755);
    assert!(directory.resolve("backup.sh").is_ok());
}

#[test]
fn a_link_that_leaves_the_directory_is_refused_as_outside() {
    let (_folder, directory) = directory();
    symlink("/bin/sh", directory.root().join("evil.sh")).unwrap();
    assert!(refusal(&directory, "evil.sh").contains("outside the scripts directory"));
}

#[test]
fn a_script_anyone_can_write_is_refused() {
    let (_folder, directory) = directory();
    file(directory.root(), "backup.sh", 0o777);
    assert!(refusal(&directory, "backup.sh").contains("written by group or others"));
}

#[test]
fn a_private_script_in_a_directory_anyone_can_write_is_refused() {
    let (_folder, directory) = directory();
    file(directory.root(), "backup.sh", 0o755);
    fs::set_permissions(directory.root(), fs::Permissions::from_mode(0o777)).unwrap();
    assert!(refusal(&directory, "backup.sh").contains("which group or others can write"));
}

#[test]
fn a_script_that_is_not_executable_is_refused() {
    let (_folder, directory) = directory();
    file(directory.root(), "notes.sh", 0o644);
    assert!(refusal(&directory, "notes.sh").contains("not executable"));
}

#[test]
fn a_directory_is_refused() {
    let (_folder, directory) = directory();
    fs::create_dir(directory.root().join("media")).unwrap();
    assert!(refusal(&directory, "media").contains("not a regular file"));
}

#[test]
fn the_listing_goes_two_levels_deep_and_says_why_a_script_cannot_run() {
    let (_folder, directory) = directory();
    file(directory.root(), "backup.sh", 0o755);
    file(directory.root(), "open.sh", 0o777);
    let nested = directory.root().join("media");
    fs::create_dir(&nested).unwrap();
    fs::set_permissions(&nested, fs::Permissions::from_mode(0o755)).unwrap();
    file(&nested, "restart.sh", 0o755);
    let deeper = nested.join("deeper");
    fs::create_dir(&deeper).unwrap();
    file(&deeper, "hidden.sh", 0o755);
    let listed = directory.tree().map(|tree| tree.files).unwrap();
    let names: Vec<&str> = listed.iter().map(|entry| entry.path.as_str()).collect();
    assert_eq!(names, vec!["backup.sh", "media/restart.sh", "open.sh"]);
    assert!(listed[0].problem.is_none());
    assert!(listed[2].problem.is_some());
}

#[test]
fn a_missing_directory_is_no_listing() {
    let folder = TempDir::new().unwrap();
    let directory = ScriptsDirectory::at(
        folder.path().join("scripts"),
        Arc::new(FileOwner::of_this_process()),
    );
    assert!(directory.tree().map(|tree| tree.files).is_none());
}

#[test]
fn a_script_deeper_than_one_subfolder_or_hidden_is_refused_and_not_listed() {
    let (_folder, directory) = directory();
    let nested = directory.root().join("media");
    fs::create_dir(&nested).unwrap();
    fs::set_permissions(&nested, fs::Permissions::from_mode(0o755)).unwrap();
    file(&nested, "restart.sh", 0o755);
    let deeper = nested.join("old");
    fs::create_dir(&deeper).unwrap();
    fs::set_permissions(&deeper, fs::Permissions::from_mode(0o755)).unwrap();
    file(&deeper, "restart.sh", 0o755);
    let hidden = directory.root().join(".cache");
    fs::create_dir(&hidden).unwrap();
    fs::set_permissions(&hidden, fs::Permissions::from_mode(0o755)).unwrap();
    file(&hidden, "tool.sh", 0o755);
    file(directory.root(), ".secret.sh", 0o755);
    use crate::types::ProblemCode;
    assert_eq!(
        directory.resolve("media/old/restart.sh").unwrap_err().code,
        ProblemCode::TooDeep
    );
    assert_eq!(
        directory.resolve(".cache/tool.sh").unwrap_err().code,
        ProblemCode::Hidden
    );
    assert_eq!(
        directory.resolve(".secret.sh").unwrap_err().code,
        ProblemCode::Hidden
    );
    let names: Vec<String> = directory
        .tree()
        .unwrap()
        .files
        .into_iter()
        .map(|entry| entry.path)
        .collect();
    assert_eq!(names, vec!["media/restart.sh"]);
}

#[test]
fn a_problem_names_its_code_and_the_path_it_concerns() {
    let (_folder, directory) = directory();
    file(directory.root(), "open.sh", 0o777);
    file(directory.root(), "notes.sh", 0o644);
    use crate::types::ProblemCode;
    let open = directory.resolve("open.sh").unwrap_err();
    assert_eq!(open.code, ProblemCode::Writable);
    assert!(open.path.ends_with("scripts/open.sh"));
    assert_eq!(
        directory.resolve("notes.sh").unwrap_err().code,
        ProblemCode::NotExecutable
    );
    fs::set_permissions(directory.root(), fs::Permissions::from_mode(0o775)).unwrap();
    file(directory.root(), "fine.sh", 0o755);
    let folder = directory.resolve("fine.sh").unwrap_err();
    assert_eq!(folder.code, ProblemCode::FolderWritable);
    assert!(folder.path.ends_with("scripts"));
}

#[test]
fn a_link_into_a_hidden_or_deep_path_is_refused() {
    let (_folder, directory) = directory();
    let hidden = directory.root().join(".cache");
    fs::create_dir(&hidden).unwrap();
    fs::set_permissions(&hidden, fs::Permissions::from_mode(0o755)).unwrap();
    file(&hidden, "tool.sh", 0o755);
    std::os::unix::fs::symlink(hidden.join("tool.sh"), directory.root().join("tool.sh")).unwrap();
    assert_eq!(
        directory.resolve("tool.sh").unwrap_err().code,
        crate::types::ProblemCode::Hidden
    );
}

mod settings {
    use toml_edit::DocumentMut;

    use crate::services::{scripts_settings, validate_scripts};

    fn document(text: &str) -> DocumentMut {
        text.parse().unwrap()
    }

    #[test]
    fn editing_is_off_unless_the_file_says_so() {
        assert!(!scripts_settings(&document("")).editing);
        assert!(scripts_settings(&document("[scripts]\nediting = true\n")).editing);
    }

    #[test]
    fn a_mistyped_key_is_named() {
        let errors = validate_scripts(&document("[scripts]\neditable = true\n"));
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "scripts.editable");
        let errors = validate_scripts(&document("[scripts]\nediting = \"yes\"\n"));
        assert_eq!(errors[0].field, "scripts.editing");
        assert!(validate_scripts(&document("[scripts]\nediting = false\n")).is_empty());
    }
}

mod writing {
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::time::{Duration, SystemTime};

    use portal_config::Revision;
    use portal_feature::ApiError;
    use portal_model::ScriptPath;

    use crate::services::ScriptWriter;

    fn writer() -> (tempfile::TempDir, ScriptWriter) {
        let folder = tempfile::tempdir().unwrap();
        let root = folder.path().join("scripts");
        let writer = ScriptWriter::at(root);
        writer.create_folder(None).unwrap();
        (folder, writer)
    }

    fn path(text: &str) -> ScriptPath {
        ScriptPath::parse(text).unwrap()
    }

    fn mode(path: &std::path::Path) -> u32 {
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn a_new_script_is_written_whole_with_mode_0700_and_no_temporary_is_left() {
        let (folder, writer) = writer();
        let revision = writer
            .create(&path("backup.sh"), "#!/bin/sh\necho hi\n")
            .unwrap();
        let file = folder.path().join("scripts/backup.sh");
        assert_eq!(fs::read_to_string(&file).unwrap(), "#!/bin/sh\necho hi\n");
        assert_eq!(mode(&file), 0o700);
        assert_eq!(mode(&folder.path().join("scripts")), 0o700);
        assert_eq!(revision, Revision::of(b"#!/bin/sh\necho hi\n"));
        let names: Vec<String> = fs::read_dir(folder.path().join("scripts"))
            .unwrap()
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, vec!["backup.sh"]);
        assert!(matches!(
            writer.create(&path("backup.sh"), "x"),
            Err(ApiError::Conflict(_))
        ));
    }

    #[test]
    fn a_replace_needs_the_revision_it_read_and_keeps_mode_0700() {
        let (folder, writer) = writer();
        let first = writer.create(&path("backup.sh"), "one\n").unwrap();
        let file = folder.path().join("scripts/backup.sh");
        fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
        let second = writer.replace(&path("backup.sh"), "two\n", &first).unwrap();
        assert_eq!(mode(&file), 0o700);
        assert!(matches!(
            writer.replace(&path("backup.sh"), "three\n", &first),
            Err(ApiError::Conflict(_))
        ));
        assert_eq!(fs::read_to_string(&file).unwrap(), "two\n");
        assert_eq!(writer.read(&path("backup.sh")).unwrap().1, second);
    }

    #[test]
    fn a_symbolic_link_is_never_written_through() {
        let (folder, writer) = writer();
        let outside = folder.path().join("outside.sh");
        fs::write(&outside, "keep\n").unwrap();
        symlink(&outside, folder.path().join("scripts/tool.sh")).unwrap();
        let revision = Revision::of(b"keep\n");
        assert!(matches!(
            writer.replace(&path("tool.sh"), "changed\n", &revision),
            Err(ApiError::Invalid(_))
        ));
        symlink(folder.path(), folder.path().join("scripts/away")).unwrap();
        assert!(matches!(
            writer.create(&path("away/x.sh"), "x"),
            Err(ApiError::Invalid(_))
        ));
        assert_eq!(fs::read_to_string(&outside).unwrap(), "keep\n");
    }

    #[test]
    fn a_folder_is_created_0700_and_removed_only_when_empty() {
        let (folder, writer) = writer();
        writer.create_folder(Some("media")).unwrap();
        assert_eq!(mode(&folder.path().join("scripts/media")), 0o700);
        writer.create(&path("media/restart.sh"), "x").unwrap();
        let Err(ApiError::Conflict(message)) = writer.delete_folder("media") else {
            panic!("a folder with a script was removed");
        };
        assert!(message.contains("restart.sh"), "{message}");
        let revision = Revision::of(b"x");
        writer
            .rename(&path("media/restart.sh"), &path("restart.sh"), &revision)
            .unwrap();
        writer.delete_folder("media").unwrap();
        writer.delete(&path("restart.sh"), &revision).unwrap();
        assert!(!folder.path().join("scripts/restart.sh").exists());
    }

    #[test]
    fn a_stale_temporary_is_swept_and_a_fresh_one_is_kept() {
        let (folder, writer) = writer();
        let stale = folder.path().join("scripts/.backup.sh.tmp-1-2-3");
        let fresh = folder.path().join("scripts/.backup.sh.tmp-4-5-6");
        fs::write(&stale, "half").unwrap();
        fs::write(&fresh, "half").unwrap();
        let old = SystemTime::now() - Duration::from_secs(2 * 60 * 60);
        fs::File::options()
            .write(true)
            .open(&stale)
            .unwrap()
            .set_modified(old)
            .unwrap();
        assert_eq!(writer.sweep(), 1);
        assert!(!stale.exists());
        assert!(fresh.exists());
    }

    #[test]
    fn content_is_bounded_and_must_be_text() {
        let (_folder, writer) = writer();
        let large = "x".repeat(256 * 1024 + 1);
        assert!(matches!(
            writer.create(&path("big.sh"), &large),
            Err(ApiError::Invalid(_))
        ));
        assert!(matches!(
            writer.create(&path("nul.sh"), "a\0b"),
            Err(ApiError::Invalid(_))
        ));
    }
}
