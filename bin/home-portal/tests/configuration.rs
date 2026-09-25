mod support;

use home_portal::registered;
use portal_config::ConfigStore;
use std::os::unix::fs::PermissionsExt;

const EXAMPLE: &str = "home-portal.example.toml";

#[test]
fn the_example_configuration_with_one_user_passes_every_validator() {
    let directory = tempfile::tempdir().unwrap();
    let path = support::with_extra(&directory, "secret", "");
    let wiring = support::wiring_for(&path);
    let registry = registered(&wiring).unwrap();
    let validators = registry
        .features
        .iter()
        .filter_map(|feature| feature.validator())
        .collect();
    wiring.configuration.adopt(validators).unwrap();
}

#[test]
fn the_example_on_its_own_refuses_to_start_and_names_the_command() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    std::fs::copy(root.join(EXAMPLE), &path).unwrap();
    let wiring = support::wiring_for(&path);
    let registry = registered(&wiring).unwrap();
    let validators = registry
        .features
        .iter()
        .filter_map(|feature| feature.validator())
        .collect();
    let store: &ConfigStore = &wiring.configuration;
    let message = store.adopt(validators).unwrap_err().to_string();
    assert!(message.contains("home-portal password-hash"), "{message}");
}

fn boot_error(extra: &str) -> String {
    let directory = tempfile::tempdir().unwrap();
    let path = support::with_extra(&directory, "secret", extra);
    let wiring = support::wiring_for(&path);
    let registry = registered(&wiring).unwrap();
    let validators = registry
        .features
        .iter()
        .filter_map(|feature| feature.validator())
        .collect();
    wiring.configuration.adopt(validators).unwrap();
    registry
        .widgets
        .validate(&wiring.configuration.read().document)
        .into_iter()
        .map(|error| format!("{}: {}", error.field, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_widget_with_a_provider_but_no_id_is_refused_at_boot() {
    let message = boot_error("\n[[dashboard.widgets]]\ntype = \"host-metrics\"\n");
    assert!(message.contains("id"), "{message}");
    assert!(message.contains("host-metrics"), "{message}");
}

#[test]
fn a_widget_setting_the_provider_refuses_names_the_widget_and_the_setting() {
    let message = boot_error(
        "\n[[dashboard.widgets]]\ntype = \"host-metrics\"\nid = \"box\"\nsettings = { disks = 5 }\n",
    );
    assert!(
        message.contains("dashboard.widgets.box.settings.disks"),
        "{message}"
    );
}

#[test]
fn a_widget_of_a_type_no_provider_serves_needs_no_id() {
    assert_eq!(
        boot_error("\n[[dashboard.widgets]]\ntype = \"traffic\"\n"),
        ""
    );
}

const DOCUMENTED: &str = r#"
[interface]
default_language = "ru"

[environments.local]
networks = ["192.168.1.0/24"]

[environments.vpn]
networks = ["10.8.0.0/24"]

[[services]]
id = "media"
name = "Media"
url = "http://192.168.1.10:8096"
addresses = { local = "http://192.168.1.10:8096", vpn = "http://10.8.0.10:8096", internet = "https://media.example.com" }
environments = ["local", "vpn"]
icon = "auto"
public = true
public_status = true
notify = true
probe = { environment = "local" }

[[dashboard.widgets]]
type = "host-metrics"
id = "box"
settings = { disks = ["/"] }

[[dashboard.widgets]]
type = "weather"
id = "riga"
public = true
settings = { latitude = 56.95, longitude = 24.11, timezone = "Europe/Riga", units = "metric", days = 3 }

[[dashboard.widgets]]
type = "calendar"
id = "home"
environments = ["local", "vpn"]
settings = { url = "https://calendar.example.com/home.ics", days = 7, limit = 10 }
"#;

#[test]
fn every_section_the_example_documents_is_accepted_when_it_is_used() {
    assert_eq!(boot_error(DOCUMENTED), "");
}

#[test]
fn the_secrets_example_is_read_from_an_include_and_never_reaches_the_document() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let directory = tempfile::tempdir().unwrap();
    let secrets = directory.path().join("secrets.toml");
    let text = std::fs::read_to_string(root.join("secrets.example.toml")).unwrap();
    std::fs::write(
        &secrets,
        format!("{text}\ntelegram_token = \"123456789:AA\"\n"),
    )
    .unwrap();
    std::fs::set_permissions(&secrets, PermissionsExt::from_mode(0o600)).unwrap();
    let path = directory.path().join("home-portal.toml");
    let hash = portal_auth::hash_password("secret").unwrap();
    std::fs::write(
        &path,
        format!(
            "include = [\"secrets.toml\"]\n\n[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n\n[notifications.telegram]\nenabled = true\nsecret = \"telegram_token\"\nchat_id = \"42\"\n"
        ),
    )
    .unwrap();
    let wiring = support::wiring_for(&path);
    let registry = registered(&wiring).unwrap();
    assert!(registry.configuration.secret("telegram_token").is_some());
    assert!(
        registry
            .configuration
            .read()
            .document
            .get("secrets")
            .is_none(),
        "the secrets table must not be part of the merged document"
    );
}

fn adopted(
    extra: &str,
) -> (
    tempfile::TempDir,
    std::path::PathBuf,
    home_portal::Wiring,
    home_portal::Registry,
) {
    let directory = tempfile::tempdir().unwrap();
    let path = support::with_extra(&directory, "secret", extra);
    let wiring = support::wiring_for(&path);
    let registry = registered(&wiring).unwrap();
    let validators = registry
        .features
        .iter()
        .filter_map(|feature| feature.validator())
        .collect();
    wiring.configuration.adopt(validators).unwrap();
    (directory, path, wiring, registry)
}

#[test]
fn the_provider_checks_refuse_a_bad_setting_through_the_store_as_boot_does() {
    let (_directory, _path, wiring, registry) = adopted(
        "\n[[dashboard.widgets]]\ntype = \"host-metrics\"\nid = \"box\"\nsettings = { disks = 5 }\n",
    );
    let message = wiring
        .configuration
        .adopt_checks(vec![registry.widgets.checker()])
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("dashboard.widgets.box.settings.disks"),
        "{message}"
    );
}

#[tokio::test]
async fn a_write_is_refused_while_the_file_holds_a_bad_widget_setting_added_by_hand() {
    let (_directory, path, wiring, registry) = adopted("");
    wiring
        .configuration
        .adopt_checks(vec![registry.widgets.checker()])
        .unwrap();
    let revision = wiring.configuration.read().revision;
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str("\n[[dashboard.widgets]]\ntype = \"host-metrics\"\nid = \"box\"\nsettings = { disks = 5 }\n");
    std::fs::write(&path, text).unwrap();
    let refused = wiring
        .configuration
        .update(&path, &revision, |_| Ok(()))
        .await
        .unwrap_err();
    let message = format!("{refused:?}");
    assert!(
        message.contains("dashboard.widgets.box.settings.disks"),
        "{message}"
    );
}

#[test]
fn an_unsupported_default_language_refuses_to_start_and_names_the_key() {
    let directory = tempfile::tempdir().unwrap();
    let path = support::with_extra(
        &directory,
        "secret",
        "\n[interface]\ndefault_language = \"de\"\n",
    );
    let wiring = support::wiring_for(&path);
    let registry = registered(&wiring).unwrap();
    let message = home_portal::adopt(&wiring.configuration, &registry)
        .unwrap_err()
        .to_string();
    assert!(message.contains("interface.default_language"), "{message}");
}
