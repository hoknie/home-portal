mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use home_portal::registered;
use portal_auth::hash_password;
use portal_services::ServicesSection;
use toml_edit::DocumentMut;

const EXAMPLES: &str = "examples";
const SPLIT: [&str; 4] = [
    "home-portal.toml",
    "services.toml",
    "widgets.toml",
    "automations.toml",
];
const SECRETS: &str = "secrets.example.toml";
const SECRET_VALUES: &str = "telegram_token = \"123456789:AA\"\ncalendar_password = \"example\"\n";
const ENVIRONMENTS: &str = "[environments.local]\nnetworks = [\"192.168.1.0/24\"]\n\n[environments.vpn]\nnetworks = [\"10.8.0.0/24\"]\n";

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(EXAMPLES)
}

fn user() -> String {
    let hash = hash_password("secret").unwrap();
    format!("\n[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n")
}

fn every_error(path: &Path) -> String {
    let wiring = support::wiring_for(path);
    let registry =
        registered(&wiring).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let validators = registry
        .features
        .iter()
        .filter_map(|feature| feature.validator())
        .collect();
    let mut problems = Vec::new();
    if let Err(error) = wiring.configuration.adopt(validators) {
        problems.push(error.to_string());
    }
    if let Err(error) = wiring
        .configuration
        .adopt_checks(vec![registry.widgets.checker()])
    {
        problems.push(error.to_string());
    }
    problems.join("\n")
}

fn services_of(path: &Path) -> Vec<portal_services::ServiceEntry> {
    let document: DocumentMut = fs::read_to_string(path).unwrap().parse().unwrap();
    ServicesSection::read(&document)
        .unwrap_or_else(|message| panic!("{}: {message}", path.display()))
        .services
}

fn unknown_keys(path: &Path) -> Vec<String> {
    let document: DocumentMut = fs::read_to_string(path).unwrap().parse().unwrap();
    let Some(tables) = document
        .get("services")
        .and_then(|item| item.as_array_of_tables())
    else {
        return Vec::new();
    };
    let mut unknown = Vec::new();
    for (table, service) in tables.iter().zip(services_of(path)) {
        let known = serde_json::to_value(&service).unwrap();
        for (key, item) in table.iter() {
            if known.get(key).is_none() {
                unknown.push(format!("{}: {}.{key}", path.display(), service.id));
            }
            if let Some(inline) = item.as_inline_table() {
                for (inner, _) in inline.iter() {
                    if known[key].is_object() && known[key].get(inner).is_none() {
                        unknown.push(format!("{}: {}.{key}.{inner}", path.display(), service.id));
                    }
                }
            }
        }
    }
    unknown
}

fn assert_teaches(path: &Path) {
    assert_eq!(unknown_keys(path), Vec::<String>::new());
    for service in services_of(path) {
        assert!(
            service
                .notes
                .as_deref()
                .is_some_and(|notes| !notes.trim().is_empty()),
            "{}: service {} has no notes",
            path.display(),
            service.id
        );
        assert!(
            service.icon.is_some(),
            "{}: service {} has no icon",
            path.display(),
            service.id
        );
    }
}

#[test]
fn the_split_example_loads_through_every_validator() {
    let directory = tempfile::tempdir().unwrap();
    let source = examples().join("split");
    for name in SPLIT {
        fs::copy(source.join(name), directory.path().join(name)).unwrap();
    }
    let main = directory.path().join("home-portal.toml");
    let text = fs::read_to_string(&main).unwrap();
    fs::write(&main, format!("{text}{}", user())).unwrap();
    let secrets = directory.path().join("secrets.toml");
    let example = fs::read_to_string(source.join(SECRETS)).unwrap();
    fs::write(&secrets, format!("{example}{SECRET_VALUES}")).unwrap();
    fs::set_permissions(&secrets, PermissionsExt::from_mode(0o600)).unwrap();
    assert_eq!(every_error(&main), "");
    assert_teaches(&source.join("services.toml"));
}

#[test]
fn every_service_example_loads_through_every_validator_and_teaches() {
    let files: Vec<PathBuf> = fs::read_dir(examples().join("services"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .collect();
    assert_eq!(files.len(), 4);
    for file in files {
        let directory = tempfile::tempdir().unwrap();
        fs::copy(&file, directory.path().join("services.toml")).unwrap();
        let main = directory.path().join("home-portal.toml");
        fs::write(
            &main,
            format!("include = [\"services.toml\"]\n\n{ENVIRONMENTS}{}", user()),
        )
        .unwrap();
        assert_eq!(every_error(&main), "", "{}", file.display());
        assert_teaches(&file);
    }
}

#[test]
fn a_misspelled_field_in_an_example_is_named_with_its_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("media.toml");
    let text = fs::read_to_string(examples().join("services/media.toml")).unwrap();
    fs::write(
        &path,
        text.replacen("probe = { path =", "probe = { pth =", 1),
    )
    .unwrap();
    let unknown = unknown_keys(&path);
    assert_eq!(unknown.len(), 1, "{unknown:?}");
    assert!(
        unknown[0].ends_with("media.toml: jellyfin.probe.pth"),
        "{unknown:?}"
    );
}

#[test]
fn the_service_catalogue_holds_every_service_it_promises() {
    let ids: Vec<String> = fs::read_dir(examples().join("services"))
        .unwrap()
        .flat_map(|entry| services_of(&entry.unwrap().path()))
        .map(|service| service.id)
        .collect();
    for promised in [
        "jellyfin",
        "plex",
        "home-assistant",
        "qbittorrent",
        "transmission",
        "pi-hole",
        "adguard",
        "nextcloud",
        "immich",
        "grafana",
        "proxmox",
        "synology",
        "router",
        "printer",
        "nas-ssh",
        "switch",
    ] {
        assert!(
            ids.iter().any(|id| id == promised),
            "no example for {promised}"
        );
    }
}

#[test]
fn the_split_layout_uses_every_size_and_every_widget_type_the_portal_serves() {
    let text = fs::read_to_string(examples().join("split/widgets.toml")).unwrap();
    for size in ["quarter", "third", "half", "two-thirds", "full"] {
        assert!(
            text.contains(&format!("size = \"{size}\"")),
            "no {size} widget"
        );
    }
    for kind in [
        "status-summary",
        "services",
        "host-metrics",
        "weather",
        "calendar",
    ] {
        assert!(
            text.contains(&format!("type = \"{kind}\"")),
            "no {kind} widget"
        );
    }
}

#[test]
fn the_main_example_points_at_the_examples_and_loads_with_a_user() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = fs::read_to_string(root.join("config/home-portal.example.toml")).unwrap();
    assert!(text.contains("../examples/README.md"));
    assert!(text.contains("../README.md"));
    assert!(root.join("README.md").is_file());
    let directory = tempfile::tempdir().unwrap();
    let path = support::with_extra(&directory, "secret", "");
    assert_eq!(every_error(&path), "");
}

#[test]
fn the_split_example_runs_its_automations_from_its_sample_script() {
    let main = examples().join("split/home-portal.toml");
    let document: DocumentMut = fs::read_to_string(examples().join("split/automations.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let scripts: Vec<&str> = document["automations"]
        .as_array_of_tables()
        .unwrap()
        .iter()
        .map(|table| table["run"]["script"].as_str().unwrap())
        .collect();
    assert_eq!(scripts.len(), 6);
    let directory = portal_automations::ScriptsDirectory::at(main.with_file_name("scripts"));
    for script in scripts {
        directory
            .resolve(script)
            .unwrap_or_else(|refusal| panic!("{script}: {}", refusal.message));
    }
}
