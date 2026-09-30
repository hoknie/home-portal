use std::io::Write;
use std::process::{Command, Output, Stdio};

const BINARY: &str = env!("CARGO_BIN_EXE_home-portal");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run(arguments: &[&str]) -> Output {
    run_with(arguments, &[])
}

fn run_with(arguments: &[&str], variables: &[(&str, &str)]) -> Output {
    let directory = tempfile::tempdir().unwrap();
    Command::new(BINARY)
        .args(arguments)
        .current_dir(directory.path())
        .env("HOME_PORTAL_CONFIG", directory.path().join("missing.toml"))
        .env_remove("CLICOLOR_FORCE")
        .env_remove("NO_COLOR")
        .envs(variables.iter().copied())
        .output()
        .unwrap()
}

fn hash_from(input: &str) -> Output {
    let mut child = Command::new(BINARY)
        .arg("password-hash")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn a_piped_password_gives_one_hash_line_on_stdout() {
    let output = hash_from("secret\n");
    assert!(output.status.success(), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert!(stdout.starts_with("$argon2id$"), "{stdout}");
}

#[test]
fn an_empty_password_fails_with_an_error_line_and_status_one() {
    let output = hash_from("\n");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error = text(&output.stderr);
    assert!(error.starts_with("error: "), "{error}");
}

#[test]
fn help_lists_every_command_with_examples_without_a_configuration() {
    let output = run(&["--help"]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    let help = text(&output.stdout);
    for word in [
        "serve",
        "password-hash",
        "probe",
        "proxy",
        "permissions",
        "--version",
        "Examples:",
        "proxy render",
        "--kind tcp",
        "HOME_PORTAL_CONFIG",
        "HOME_PORTAL_ADDRESS",
    ] {
        assert!(help.contains(word), "{word} missing from\n{help}");
    }
}

#[test]
fn the_help_command_says_the_same_as_the_help_flag() {
    assert_eq!(run(&["help"]).stdout, run(&["--help"]).stdout);
}

#[test]
fn probe_help_lists_the_kinds_and_an_example() {
    for arguments in [&["probe", "--help"][..], &["help", "probe"][..]] {
        let output = run(arguments);
        assert!(output.status.success());
        let help = text(&output.stdout);
        for word in [
            "<SERVICE|URL>",
            "--kind",
            "http",
            "tcp",
            "icmp",
            "Examples:",
        ] {
            assert!(help.contains(word), "{word} missing from\n{help}");
        }
    }
}

#[test]
fn every_command_has_help_with_an_example() {
    for command in ["password-hash", "probe", "proxy", "permissions"] {
        let output = run(&[command, "--help"]);
        assert!(output.status.success(), "{command}");
        assert!(text(&output.stdout).contains("Examples:"), "{command}");
    }
}

#[test]
fn the_version_flags_print_the_name_and_version() {
    for flag in ["--version", "-V"] {
        let output = run(&[flag]);
        assert!(output.status.success());
        assert_eq!(text(&output.stdout), format!("home-portal {VERSION}\n"));
    }
}

#[test]
fn a_misspelled_command_is_a_usage_error_that_suggests_the_right_one() {
    let output = run(&["prob", "media"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let error = text(&output.stderr);
    for word in ["'prob'", "'probe'", "Usage:", "--help"] {
        assert!(error.contains(word), "{word} missing from\n{error}");
    }
}

#[test]
fn a_probe_without_a_target_shows_its_usage() {
    let output = run(&["probe"]);
    assert_eq!(output.status.code(), Some(2));
    let error = text(&output.stderr);
    assert!(error.contains("Usage: home-portal probe"), "{error}");
}

#[test]
fn an_unknown_proxy_action_names_render() {
    let output = run(&["proxy", "show"]);
    assert_eq!(output.status.code(), Some(2));
    let error = text(&output.stderr);
    assert!(error.contains("'show'"), "{error}");
    assert!(error.contains("render"), "{error}");
}

#[test]
fn piped_help_and_errors_carry_no_escape_sequences() {
    for arguments in [&["--help"][..], &["prob"][..]] {
        let output = run(arguments);
        assert!(!output.stdout.contains(&0x1b));
        assert!(!output.stderr.contains(&0x1b));
    }
}

#[test]
fn no_color_keeps_help_plain_and_forced_colour_styles_it() {
    let plain = run_with(&["--help"], &[("NO_COLOR", "1")]);
    assert!(!plain.stdout.contains(&0x1b));
    let forced = run_with(&["--help"], &[("CLICOLOR_FORCE", "1")]);
    assert!(forced.stdout.contains(&0x1b));
}
