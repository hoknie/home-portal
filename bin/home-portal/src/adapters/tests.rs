use std::fs;
use std::sync::Arc;

use portal_automations::Directory;
use portal_config::ConfigStore;
use portal_feature::{EventSink, PortalEvent};
use portal_model::Environment;
use portal_proxy::PublishedServices;
use portal_services::{Publishing, ServicesFeature, ServicesPorts};

use super::{
    AutomationDirectory, PortalProcess, ProxyPublishing, ScriptShelf, ServicePublications,
};

struct Silent;

#[async_trait::async_trait]
impl EventSink for Silent {
    fn emit(&self, _event: PortalEvent) {}

    async fn settle(&self, _within: std::time::Duration) {}
}

const FILE: &str = "[environments.local]\nnetworks = [\"192.168.0.0/16\"]\n\n[[services]]\nid = \"media\"\nname = \"Media\"\nurl = \"http://media.lan:8096\"\naddresses = { local = \"http://192.168.1.10:8096\" }\nproxy = { host = \"media.example.com\" }\nprobe = { enabled = false }\n\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"https://192.168.1.5:5001\"\nproxy = { host = \"nas.example.com\" }\nprobe = { enabled = false }\n\n[[services]]\nid = \"printer\"\nname = \"Printer\"\nurl = \"http://192.168.1.30\"\nprobe = { enabled = false }\n";

fn publishing_of(store: &Arc<ConfigStore>) -> ProxyPublishing {
    ProxyPublishing::new(
        portal_proxy::CurrentProxySettings::new(store.clone()),
        portal_proxy::CheckPublication::new(store.clone()),
    )
}

fn services(text: &str) -> (tempfile::TempDir, Arc<ConfigStore>, Arc<ServicesFeature>) {
    let (directory, path) = portal_testing::written(text);
    let store = Arc::new(portal_testing::opened(&path).unwrap());
    let services = ServicesFeature::new(
        store.clone(),
        Environment::parse("local").unwrap(),
        ServicesPorts {
            observers: Vec::new(),
            publishing: Arc::new(publishing_of(&store)),
            events: Arc::new(Silent),
        },
    )
    .unwrap();
    (directory, store, Arc::new(services))
}

#[test]
fn a_published_service_is_proxied_to_the_address_it_is_probed_at() {
    let (_directory, _, services) = services(FILE);
    let published = ServicePublications { services }.published();
    let upstreams: Vec<(&str, &str)> = published
        .iter()
        .map(|service| (service.id.as_str(), service.upstream.as_str()))
        .collect();
    assert_eq!(
        upstreams,
        vec![
            ("media", "http://192.168.1.10:8096"),
            ("nas", "https://192.168.1.5:5001")
        ]
    );
}

#[test]
fn publishing_follows_the_proxy_section() {
    let (_directory, store, _) = services(FILE);
    let publishing = publishing_of(&store);
    assert_eq!(publishing.https_port(), None);
    let (_directory, store, _) = services(&format!(
        "[modules]\nproxy = true\n\n[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[proxy]\nportal_host = \"portal.example.com\"\n\n{FILE}"
    ));
    assert_eq!(publishing_of(&store).https_port(), Some(443));
}

#[test]
fn publishing_names_a_sign_in_the_cookie_domain_cannot_reach_by_the_service_field() {
    let (_directory, store, _) = services(&format!("{PROXIED}\n{FILE}"));
    let publication = portal_model::Publication {
        host: "nas.example.org".into(),
        environments: vec!["internet".into()],
        auth: vec!["internet".into()],
        tls: None,
        upstream_verify: true,
    };
    let errors = publishing_of(&store).problems(&publication);
    let fields: Vec<&str> = errors.iter().map(|error| error.field.as_str()).collect();
    assert_eq!(fields, vec!["proxy.auth"]);
}

fn connection(text: &str) -> (tempfile::TempDir, super::NetworkConnection) {
    let (directory, path) = portal_testing::written(text);
    let configuration = Arc::new(portal_testing::opened(&path).unwrap());
    (
        directory,
        super::NetworkConnection {
            network: portal_network::CurrentNetwork::new(configuration.clone()),
            proxy: portal_proxy::CurrentProxySettings::new(configuration),
        },
    )
}

const PROXIED: &str = "[modules]\nproxy = true\n\n[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[proxy]\nportal_host = \"portal.example.com\"\ncookie_domain = \"example.com\"\n";

fn through(host: &str) -> axum::http::HeaderMap {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("x-forwarded-host", host.parse().unwrap());
    headers
}

fn peer(address: &str) -> Option<std::net::SocketAddr> {
    Some(format!("{address}:40000").parse().unwrap())
}

#[test]
fn one_sign_in_through_caddy_covers_every_published_host_under_the_cookie_domain() {
    use portal_auth::Connection;
    let (_directory, connection) = connection(PROXIED);
    let scope = connection.cookie_scope(peer("127.0.0.1"), &through("portal.example.com:443"));
    assert_eq!(scope.domain.as_deref(), Some("example.com"));
    assert!(scope.secure);
}

#[test]
fn a_sign_in_made_directly_while_the_proxy_is_on_keeps_a_plain_cookie() {
    use portal_auth::Connection;
    let (_directory, connection) = connection(PROXIED);
    let direct = connection.cookie_scope(peer("127.0.0.1"), &axum::http::HeaderMap::new());
    assert_eq!(direct, portal_auth::CookieScope::default());
    let untrusted = connection.cookie_scope(peer("192.168.1.40"), &through("portal.example.com"));
    assert_eq!(untrusted, portal_auth::CookieScope::default());
    let elsewhere = connection.cookie_scope(peer("127.0.0.1"), &through("portal.example.org"));
    assert_eq!(elsewhere, portal_auth::CookieScope::default());
}

#[test]
fn without_the_proxy_the_cookie_has_no_domain() {
    use portal_auth::Connection;
    let (_directory, connection) = connection(&PROXIED.replace("proxy = true", "proxy = false"));
    let scope = connection.cookie_scope(peer("127.0.0.1"), &through("portal.example.com"));
    assert_eq!(scope, portal_auth::CookieScope::default());
}

#[test]
fn only_a_trusted_proxy_is_trusted_as_a_peer() {
    use portal_proxy::TrustedPeers;
    let (_directory, connection) = connection(PROXIED);
    assert!(connection.trusts("127.0.0.1".parse().unwrap()));
    assert!(!connection.trusts("192.168.1.40".parse().unwrap()));
}

#[test]
fn the_automation_directory_lists_services_users_and_environments_with_internet() {
    let text = format!(
        "{FILE}\n[[users]]\nname = \"admin\"\npassword_hash = \"x\"\n\n[[users]]\nname = \"guest\"\npassword_hash = \"y\"\n"
    );
    let (_directory, store, _) = services(&text);
    let directory = AutomationDirectory {
        services: portal_services::ServiceEntries::new(store.clone()),
        environments: portal_network::CurrentEnvironments::new(store.clone()),
        users: portal_auth::UserNames::new(store),
    };
    let services: Vec<String> = directory
        .services()
        .into_iter()
        .map(|choice| format!("{}={}", choice.id, choice.name))
        .collect();
    assert_eq!(services, vec!["media=Media", "nas=NAS", "printer=Printer"]);
    assert_eq!(directory.users(), vec!["admin", "guest"]);
    assert_eq!(directory.environments(), vec!["local", "internet"]);
}

async fn answering_http() -> u16 {
    portal_testing::FakeHttp::always(portal_testing::Answer::status(200).with_body("ok"))
        .await
        .address
        .port()
}

#[tokio::test]
async fn workflow_actions_wait_for_their_features_then_probe_read_and_refuse_telegram() {
    use portal_automations::PortalActions;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(
        &path,
        format!(
            "[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://127.0.0.1:{}\"\n",
            answering_http().await
        ),
    )
    .unwrap();
    let store = Arc::new(portal_testing::opened(&path).unwrap());
    let actions = super::WorkflowActions::default();
    assert_eq!(
        actions.probe("nas").await.unwrap_err(),
        super::WorkflowActions::NOT_READY
    );
    let services = ServicesFeature::new(
        store.clone(),
        Environment::internet(),
        ServicesPorts {
            observers: Vec::new(),
            publishing: Arc::new(publishing_of(&store)),
            events: Arc::new(Silent),
        },
    )
    .unwrap();
    let notifications = portal_notification::NotificationFeature::new(
        store,
        vec![Arc::new(
            portal_telegram::TelegramChannel::new("http://127.0.0.1:9").unwrap(),
        )],
    )
    .unwrap();
    let _ = actions.probe.set(services.probe_service());
    let _ = actions.status.set(services.current_status());
    let _ = actions.notify.set(notifications.send_notification());
    let probed = actions.probe("nas").await.unwrap();
    assert_eq!(probed.state, "up");
    assert_eq!(actions.status("nas").await.unwrap().state, "up");
    assert!(actions.status("ghost").await.is_err());
    let refused = actions
        .notify(Some("telegram"), "", "hello")
        .await
        .unwrap_err();
    assert!(refused.contains("not configured"), "{refused}");
}

#[tokio::test]
async fn the_portal_state_waits_for_the_portal_to_start() {
    use portal_automations::PortalActions;
    let actions = super::WorkflowActions::default();
    assert_eq!(
        actions.state().await.unwrap_err(),
        super::WorkflowActions::NOT_READY
    );
}

#[test]
fn the_script_shelf_refuses_a_link_that_leaves_the_directory_through_the_port() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    use portal_automations::{RefusalCode, ScriptLibrary};
    use portal_scripts::{ListScripts, ResolveScript};

    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("scripts");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(root.join("backup.sh"), "#!/bin/sh\n").unwrap();
    fs::set_permissions(root.join("backup.sh"), fs::Permissions::from_mode(0o755)).unwrap();
    symlink("/bin/sh", root.join("evil.sh")).unwrap();
    let main = directory.path().join("home-portal.toml");
    fs::write(&main, "[scripts]\nediting = true\n").unwrap();
    let shelf = ScriptShelf {
        resolve: ResolveScript::at(root.clone(), Arc::new(PortalProcess)),
        list: ListScripts::at(root.clone(), Arc::new(PortalProcess)),
        editing: portal_scripts::ScriptEditing::new(Arc::new(
            portal_testing::opened(&main).unwrap(),
        )),
    };
    let refusal = shelf.resolve("evil.sh").unwrap_err();
    assert_eq!(refusal.code, RefusalCode::Outside);
    assert!(shelf.resolve("backup.sh").is_ok());
    let listed = shelf.list().unwrap();
    let evil = listed.iter().find(|entry| entry.path == "evil.sh").unwrap();
    assert_eq!(evil.problem.as_ref().unwrap().code, RefusalCode::Outside);
    assert_eq!(shelf.root(), fs::canonicalize(&root).unwrap());
    assert!(shelf.editing());
}
