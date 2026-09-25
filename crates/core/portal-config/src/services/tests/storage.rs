use portal_feature::ApiError;
use toml_edit::value;

use super::support::{Portal, open};
use crate::types::{ConfigError, Storage};

#[test]
fn without_a_storage_section_everything_lies_beside_the_configuration() {
    let portal = Portal::with(&[("home-portal.toml", "port = 1\n")]);
    for kind in Storage::ALL {
        assert_eq!(
            portal.store.storage(kind),
            portal.path(kind.default_name()),
            "{}",
            kind.key()
        );
    }
    assert_eq!(
        portal.store.storage(Storage::Sessions),
        portal.path("sessions.json")
    );
}

#[test]
fn a_directory_moves_everything_and_a_named_place_wins_over_it() {
    let portal = Portal::with(&[(
        "home-portal.toml",
        "[storage]\ndirectory = \"../env\"\nscripts = \"tools\"\ncaddy = \"/opt/caddy\"\n",
    )]);
    let base = portal.directory.path();
    assert_eq!(
        portal.store.storage(Storage::History),
        base.join("../env/history")
    );
    assert_eq!(
        portal.store.storage(Storage::Sessions),
        base.join("../env/sessions.json")
    );
    assert_eq!(portal.store.storage(Storage::Scripts), base.join("tools"));
    assert_eq!(
        portal.store.storage(Storage::Caddy),
        std::path::PathBuf::from("/opt/caddy")
    );
}

#[test]
fn an_unknown_key_or_an_empty_path_stops_the_start() {
    for text in [
        "[storage]\nhistroy = \"x\"\n",
        "[storage]\nicons = \"\"\n",
        "[storage]\nicons = 3\n",
        "storage = \"env\"\n",
    ] {
        let error = open(&[("home-portal.toml", text)]).map(|_| ()).unwrap_err();
        assert!(
            matches!(error, ConfigError::Invalid { .. }),
            "{text}: {error}"
        );
        assert!(error.to_string().contains("storage"), "{error}");
    }
}

#[tokio::test]
async fn an_edit_that_breaks_the_storage_section_is_refused() {
    let portal = Portal::with(&[("home-portal.toml", "[storage]\ndirectory = \"env\"\n")]);
    let revision = portal.store.read().revision;
    let refused = portal
        .store
        .update(&portal.path("home-portal.toml"), &revision, |document| {
            document["storage"]["histroy"] = value("x");
            Ok(())
        })
        .await;
    assert!(matches!(refused, Err(ApiError::Invalid(_))), "{refused:?}");
}

#[test]
fn a_scripts_place_that_overlaps_a_written_place_stops_the_start() {
    for text in [
        "[storage]\ndirectory = \"env\"\nscripts = \"env\"\n",
        "[storage]\nscripts = \".\"\n",
        "[storage]\ndirectory = \"env\"\nscripts = \"env/caddy/bin\"\n",
        "[storage]\nhistory = \"tools/history\"\nscripts = \"tools/./x/..\"\n",
    ] {
        let error = open(&[("home-portal.toml", text)]).map(|_| ()).unwrap_err();
        assert!(
            error.to_string().contains("storage.scripts"),
            "{text}: {error}"
        );
    }
    assert!(
        open(&[(
            "home-portal.toml",
            "[storage]\ndirectory = \"../env\"\nscripts = \"../scripts\"\n"
        )])
        .is_ok()
    );
}
