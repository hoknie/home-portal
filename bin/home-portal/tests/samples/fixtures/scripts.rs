use portal_model::ScriptHeader;
use serde_json::{Value, json};

use crate::check;

fn cases() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "a shell header",
            "#!/bin/sh\n# @description Restart a service's container\n# @arg service <text> Service id\n# @arg --retries <number=3> Tries before giving up\n# @arg --force Skip the health check\nset -eu\n",
        ),
        (
            "a choice",
            "# @arg mode <fast|full=fast> How deep to check\n",
        ),
        (
            "the header ends at the first command",
            "# @arg name <text>\nset -e\n# @arg late <text>\n",
        ),
        (
            "a flag with a default",
            "#!/bin/sh\n# @arg --force <flag=true> Always\n",
        ),
        (
            "an unknown type",
            "# @arg --keep <weird> Days\n# @arg name\n",
        ),
        (
            "slashes, a second description, a repeated and a late required name",
            "// @description One\n// @description Two\n// @arg host?\n// @arg host\n// @arg port <number>\n",
        ),
        ("a positional flag", "# @arg force <flag>\n"),
        (
            "a number default that is not a number",
            "# @arg count <number=many>\n",
        ),
        (
            "a choice default that is not a choice",
            "# @arg mode <a|b=c>\n",
        ),
        ("an unclosed type", "# @arg mode <text\n"),
        ("a bad name", "# @arg Mode\n# @arg --2x\n# @arg -x\n"),
        ("an optional option mark", "# @arg --force?\n"),
        ("no header", "echo hi\n# @arg late\n"),
        ("an empty choice", "# @arg mode <a||b>\n"),
        (
            "blank lines and other tags",
            "#!/usr/bin/env python3\n\n# @author me\n#\n# @arg path? <text=/tmp> Where to look\n",
        ),
    ]
}

fn header_value(header: &ScriptHeader) -> Value {
    json!({
        "description": header.description,
        "arguments": header.arguments.iter().map(|argument| json!({
            "name": argument.name,
            "option": argument.option,
            "required": argument.required,
            "type": argument.kind.name(),
            "choices": argument.kind.choices(),
            "default": argument.default,
            "description": argument.description,
        })).collect::<Vec<_>>(),
        "problems": header.problems.iter().map(|problem| json!({
            "line": problem.line,
            "message": problem.message,
        })).collect::<Vec<_>>(),
    })
}

#[test]
fn script_header_fixtures_are_shared_with_the_interface() {
    let cases: Vec<Value> = cases()
        .into_iter()
        .map(|(name, text)| {
            json!({"name": name, "text": text, "header": header_value(&ScriptHeader::parse(text))})
        })
        .collect();
    check("script-headers", json!({ "cases": cases }));
}

fn entry(
    path: &str,
    header: &str,
    unreadable: Option<portal_scripts::Unreadable>,
) -> portal_scripts::ScriptFile {
    portal_scripts::ScriptFile {
        path: path.into(),
        problem: None,
        header: ScriptHeader::parse(header),
        size: header.len() as u64,
        modified: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_790_000_000)),
        mode: 0o700,
        unreadable,
        revision: Some(portal_config::Revision::of(header.as_bytes()).to_string()),
    }
}

#[test]
fn the_scripts_samples_match_their_serializers() {
    use portal_scripts::{ScriptFileResponse, ScriptTextResponse, ScriptTreeResponse, Unreadable};

    let restart = "#!/bin/sh\n# @description Restart a service's container\n# @arg service <text> Service id\n# @arg --force Skip the health check\nexec true\n";
    let files = [
        entry("backup.sh", "#!/bin/sh\n", None),
        entry("media/restart.sh", restart, None),
        entry("tool", "", Some(Unreadable::Binary)),
    ];
    check(
        "scripts-tree",
        serde_json::to_value(ScriptTreeResponse {
            directory: "/srv/home-portal/scripts".into(),
            exists: true,
            user_id: 501,
            left_out: 1,
            inside: true,
            folders: vec!["media".into()],
            files: files.iter().map(ScriptFileResponse::of).collect(),
        })
        .unwrap(),
    );
    check(
        "script-text",
        serde_json::to_value(ScriptTextResponse {
            path: "media/restart.sh".into(),
            content: restart.into(),
            revision: portal_config::Revision::of(restart.as_bytes()).to_string(),
            entry: ScriptFileResponse::of(&files[1]),
        })
        .unwrap(),
    );
}
