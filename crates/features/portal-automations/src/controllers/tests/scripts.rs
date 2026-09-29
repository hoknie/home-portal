use std::fs;
use std::os::unix::fs::PermissionsExt;

use axum::http::StatusCode;

use super::automations::{FILE, api, get, send};
use crate::features::AutomationsFeature;

#[tokio::test]
async fn the_scripts_carry_their_declared_arguments_and_whether_editing_is_on() {
    let api = api(FILE);
    let restart = api.folder.path().join("scripts/restart.sh");
    fs::write(
        &restart,
        "#!/bin/sh\n# @description Restart a container\n# @arg service <text> Service id\n# @arg --keep <weird> Days\nexec true\n",
    )
    .unwrap();
    fs::set_permissions(&restart, fs::Permissions::from_mode(0o755)).unwrap();
    let (status, _, body) = send(&api, get(AutomationsFeature::SCRIPTS)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["editing"], false);
    let entry = body["scripts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["path"] == "restart.sh")
        .unwrap();
    assert_eq!(entry["runnable"], true);
    assert_eq!(entry["description"], "Restart a container");
    assert_eq!(entry["arguments"][0]["name"], "service");
    assert_eq!(entry["arguments"][0]["type"], "text");
    assert_eq!(entry["arguments"][0]["required"], true);
    assert_eq!(entry["arguments"].as_array().unwrap().len(), 1);
    assert_eq!(entry["argument_problems"][0]["line"], 4);
    assert!(
        entry["argument_problems"][0]["message"]
            .as_str()
            .unwrap()
            .contains("weird is not a known type")
    );
}
