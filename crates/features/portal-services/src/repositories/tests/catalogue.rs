use toml_edit::DocumentMut;

use crate::repositories::{append, position, remove, replace};
use crate::types::{ProbeKind, ServiceEntry};

const BEFORE: &str = r#"# services I run at home

# media first
[[services]]
id = "media"
name = "Media"
url = "http://10.0.0.5:8096"

# the NAS, keep it last
[[services]]
id = "nas"
name = "NAS"
url = "http://10.0.0.6"
"#;

fn entry(id: &str, name: &str, url: &str) -> ServiceEntry {
    ServiceEntry::new(id, name, url)
}

fn document() -> DocumentMut {
    BEFORE.parse().unwrap()
}

#[test]
fn a_service_is_found_by_its_id() {
    assert_eq!(position(&document(), "nas"), Some(1));
    assert_eq!(position(&document(), "missing"), None);
}

#[test]
fn appending_a_service_keeps_every_comment_and_adds_one_block_at_the_end() {
    let mut document = document();
    append(&mut document, &entry("router", "Router", "http://10.0.0.1"));
    let expected = format!(
        "{BEFORE}\n[[services]]\nid = \"router\"\nname = \"Router\"\nurl = \"http://10.0.0.1\"\n"
    );
    assert_eq!(document.to_string(), expected);
}

#[test]
fn editing_a_service_changes_only_its_keys_and_keeps_its_comment() {
    let mut document = document();
    let mut changed = entry("media", "Media box", "http://10.0.0.5:8096");
    changed.group = Some("Media".into());
    changed.probe.every_seconds = 10;
    replace(&mut document, 0, &changed);
    let expected = BEFORE.replace(
        "name = \"Media\"\nurl = \"http://10.0.0.5:8096\"\n",
        "name = \"Media box\"\nurl = \"http://10.0.0.5:8096\"\ngroup = \"Media\"\nprobe = { every_seconds = 10 }\n",
    );
    assert_eq!(document.to_string(), expected);
}

#[test]
fn renaming_a_service_rewrites_its_id_in_place() {
    let mut document = document();
    replace(
        &mut document,
        1,
        &entry("storage", "NAS", "http://10.0.0.6"),
    );
    assert_eq!(
        document.to_string(),
        BEFORE.replace("id = \"nas\"", "id = \"storage\"")
    );
}

#[test]
fn deleting_a_service_removes_its_block_and_keeps_the_others() {
    let mut document = document();
    remove(&mut document, 0);
    let text = document.to_string();
    assert!(!text.contains("id = \"media\""));
    assert_eq!(
        text,
        "# services I run at home\n\n# the NAS, keep it last\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://10.0.0.6\"\n"
    );
}

#[test]
fn the_first_service_creates_the_section() {
    let mut document: DocumentMut = "[network]\nport = 8080\n".parse().unwrap();
    append(&mut document, &entry("media", "Media", "http://10.0.0.5"));
    assert_eq!(position(&document, "media"), Some(0));
}

#[test]
fn deleting_the_last_service_keeps_the_comment_that_opened_the_file() {
    let mut document: DocumentMut =
        "# my portal\n\n# only one\n[[services]]\nid = \"a\"\nname = \"A\"\nurl = \"http://a\"\n"
            .parse()
            .unwrap();
    remove(&mut document, 0);
    assert!(document.to_string().contains("# my portal"));
    assert!(!document.to_string().contains("# only one"));
}

#[test]
fn a_service_with_addresses_and_visibility_is_written_whole() {
    let mut document: DocumentMut = "# my portal\n\n[network]\nport = 8080\n".parse().unwrap();
    let mut service = entry("nas", "NAS", "http://nas.local");
    service
        .addresses
        .insert("local".into(), "http://192.168.1.10".into());
    service
        .addresses
        .insert("internet".into(), "https://nas.example.com".into());
    service.environments = Some(vec!["local".into(), "vpn".into()]);
    service.public = true;
    service.public_status = true;
    service.notify = Some(false);
    service.probe.environment = Some("local".into());
    append(&mut document, &service);
    assert_eq!(
        document.to_string(),
        "# my portal\n\n[network]\nport = 8080\n\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://nas.local\"\naddresses = { internet = \"https://nas.example.com\", local = \"http://192.168.1.10\" }\nenvironments = [\"local\", \"vpn\"]\npublic = true\npublic_status = true\nnotify = false\nprobe = { environment = \"local\" }\n"
    );
}

#[test]
fn clearing_the_new_fields_removes_their_keys_and_keeps_the_comment() {
    let text = "# the NAS\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://nas.local\"\naddresses = { local = \"http://192.168.1.10\" }\nenvironments = [\"local\"]\npublic = true\n";
    let mut document: DocumentMut = text.parse().unwrap();
    replace(&mut document, 0, &entry("nas", "NAS", "http://nas.local"));
    assert_eq!(
        document.to_string(),
        "# the NAS\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://nas.local\"\n"
    );
}

#[test]
fn adding_a_tcp_service_writes_its_kind_and_port_and_nothing_else_of_the_probe() {
    let mut document = document();
    let mut printer = entry("printer", "Printer", "tcp://printer.home.lan");
    printer.probe.kind = ProbeKind::Tcp;
    printer.probe.port = Some(9100);
    append(&mut document, &printer);
    let expected = format!(
        "{BEFORE}\n[[services]]\nid = \"printer\"\nname = \"Printer\"\nurl = \"tcp://printer.home.lan\"\nprobe = {{ kind = \"tcp\", port = 9100 }}\n"
    );
    assert_eq!(document.to_string(), expected);
}

#[test]
fn switching_a_service_to_icmp_keeps_its_comment_and_drops_nothing_else() {
    let mut document = document();
    let mut nas = entry("nas", "NAS", "http://10.0.0.6");
    nas.probe.kind = ProbeKind::Icmp;
    replace(&mut document, 1, &nas);
    let expected = BEFORE.replace(
        "url = \"http://10.0.0.6\"\n",
        "url = \"http://10.0.0.6\"\nprobe = { kind = \"icmp\" }\n",
    );
    assert_eq!(document.to_string(), expected);
}

#[test]
fn links_notes_and_widgets_are_written_readably_and_read_back_unchanged() {
    let mut document = document();
    let mut media = entry("media", "Media", "http://10.0.0.5:8096");
    media.links = vec![crate::types::ServiceLink {
        title: "Admin".into(),
        url: "http://10.0.0.5:8096/web/#/dashboard".into(),
    }];
    media.notes = Some("Films live on the NAS.\nRestart with `docker restart jellyfin`.".into());
    media.widgets = vec!["nas-disks".into()];
    replace(&mut document, 0, &media);
    let expected = BEFORE.replace(
        "url = \"http://10.0.0.5:8096\"\n",
        "url = \"http://10.0.0.5:8096\"\nlinks = [{ title = \"Admin\", url = \"http://10.0.0.5:8096/web/#/dashboard\" }]\nnotes = '''\nFilms live on the NAS.\nRestart with `docker restart jellyfin`.'''\nwidgets = [\"nas-disks\"]\n",
    );
    assert_eq!(document.to_string(), expected);
    let reread: DocumentMut = document.to_string().parse().unwrap();
    let section = crate::types::ServicesSection::read(&reread).unwrap();
    assert_eq!(section.services[0].notes, media.notes);
    assert_eq!(section.services[0].links, media.links);
}

#[test]
fn notes_that_cannot_be_a_literal_block_are_written_escaped() {
    let mut document = document();
    let mut media = entry("media", "Media", "http://10.0.0.5:8096");
    media.notes = Some("a '''quoted''' line\nand more".into());
    replace(&mut document, 0, &media);
    let reread: DocumentMut = document.to_string().parse().unwrap();
    let section = crate::types::ServicesSection::read(&reread).unwrap();
    assert_eq!(section.services[0].notes, media.notes);
}

#[test]
fn a_publication_is_written_as_one_inline_table_without_its_defaults() {
    let mut document = document();
    let mut published = entry("media", "Media", "http://10.0.0.5:8096");
    let mut publication = portal_model::Publication::new("media.example.com");
    publication.auth = vec!["internet".into()];
    published.proxy = Some(publication);
    replace(&mut document, 0, &published);
    let expected = BEFORE.replace(
        "url = \"http://10.0.0.5:8096\"\n",
        "url = \"http://10.0.0.5:8096\"\nproxy = { host = \"media.example.com\", auth = [\"internet\"] }\n",
    );
    assert_eq!(document.to_string(), expected);
    let read: toml_edit::DocumentMut = document.to_string().parse().unwrap();
    let section = crate::types::ServicesSection::read(&read).unwrap();
    assert_eq!(section.services[0].proxy, published.proxy);
}

#[test]
fn a_publication_with_every_field_is_written_whole() {
    let mut document = document();
    let mut published = entry("nas", "NAS", "http://10.0.0.6");
    published.proxy = Some(portal_model::Publication {
        host: "nas.home.arpa".into(),
        upstream: Some("https://10.0.0.6:5001".into()),
        environments: vec!["local".into(), "internet".into()],
        auth: Vec::new(),
        tls: Some(portal_model::TlsPolicy {
            mode: portal_model::TlsMode::Internal,
            ..portal_model::TlsPolicy::default()
        }),
        upstream_verify: false,
    });
    replace(&mut document, 1, &published);
    assert!(document.to_string().ends_with(
        "url = \"http://10.0.0.6\"\nproxy = { host = \"nas.home.arpa\", upstream = \"https://10.0.0.6:5001\", environments = [\"local\", \"internet\"], tls = { mode = \"internal\" }, upstream_verify = false }\n"
    ));
}

#[test]
fn removing_a_publication_drops_its_key_and_keeps_the_comment() {
    let mut document = document();
    let mut published = entry("media", "Media", "http://10.0.0.5:8096");
    published.proxy = Some(portal_model::Publication::new("media.example.com"));
    replace(&mut document, 0, &published);
    replace(
        &mut document,
        0,
        &entry("media", "Media", "http://10.0.0.5:8096"),
    );
    assert_eq!(document.to_string(), BEFORE);
}
