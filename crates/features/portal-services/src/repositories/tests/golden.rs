use portal_model::{Publication, TlsPolicy};
use toml_edit::DocumentMut;

use crate::repositories::{append, replace};
use crate::types::ServiceEntry;

const BEFORE: &str = r#"# services I run at home

# media first
[[services]]
id = "media"
name = "Media"
url = "http://10.0.0.5:8096"
group = "Watch" # the group
notes = "old"
"#;

fn blank() -> ServiceEntry {
    ServiceEntry {
        group: Some(String::new()),
        icon: Some("film".into()),
        description: Some(String::new()),
        notes: Some("   \n ".into()),
        ..ServiceEntry::new("router", "Router", "http://10.0.0.1")
    }
}

fn texts() -> ServiceEntry {
    ServiceEntry {
        group: Some(" ".into()),
        description: Some(" a box ".into()),
        notes: Some("first line\nsecond line\n".into()),
        proxy: Some(Publication {
            host: "media.example.com".into(),
            environments: Publication::default_environments(),
            auth: Vec::new(),
            tls: Some(TlsPolicy {
                email: Some(" ".into()),
                ..TlsPolicy::default()
            }),
            upstream_verify: true,
        }),
        ..ServiceEntry::new("media", "Media", "http://10.0.0.5:8096")
    }
}

fn appended(entry: &ServiceEntry) -> String {
    let mut document: DocumentMut = BEFORE.parse().unwrap();
    append(&mut document, &entry.normalized());
    document.to_string()
}

fn replaced(entry: &ServiceEntry) -> String {
    let mut document: DocumentMut = BEFORE.parse().unwrap();
    replace(&mut document, 0, &entry.normalized());
    document.to_string()
}

fn cases() -> Vec<(&'static str, String)> {
    vec![
        ("blank_appended", appended(&blank())),
        ("blank_replaced", replaced(&blank())),
        ("texts_appended", appended(&texts())),
        ("texts_replaced", replaced(&texts())),
    ]
}

const EXPECTED: [(&str, &str); 4] = [
    (
        "blank_appended",
        r#"# services I run at home

# media first
[[services]]
id = "media"
name = "Media"
url = "http://10.0.0.5:8096"
group = "Watch" # the group
notes = "old"

[[services]]
id = "router"
name = "Router"
url = "http://10.0.0.1"
icon = "film"
"#,
    ),
    (
        "blank_replaced",
        r#"# services I run at home

# media first
[[services]]
id = "router"
name = "Router"
url = "http://10.0.0.1"
icon = "film"
"#,
    ),
    (
        "texts_appended",
        r#"# services I run at home

# media first
[[services]]
id = "media"
name = "Media"
url = "http://10.0.0.5:8096"
group = "Watch" # the group
notes = "old"

[[services]]
id = "media"
name = "Media"
url = "http://10.0.0.5:8096"
group = " "
description = " a box "
notes = '''
first line
second line
'''
proxy = { host = "media.example.com", tls = { mode = "acme", email = " " } }
"#,
    ),
    (
        "texts_replaced",
        r#"# services I run at home

# media first
[[services]]
id = "media"
name = "Media"
url = "http://10.0.0.5:8096"
group = " "
notes = '''
first line
second line
'''
description = " a box "
proxy = { host = "media.example.com", tls = { mode = "acme", email = " " } }
"#,
    ),
];

#[test]
fn service_writes_keep_their_bytes() {
    let cases = cases();
    assert_eq!(cases.len(), EXPECTED.len());
    for ((name, text), (expected_name, expected)) in cases.iter().zip(EXPECTED) {
        assert_eq!(*name, expected_name);
        assert_eq!(text, expected, "{name}");
    }
}
