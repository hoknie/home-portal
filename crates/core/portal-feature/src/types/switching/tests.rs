use toml_edit::DocumentMut;

use super::{Module, ModuleSwitches};

fn switches(text: &str) -> ModuleSwitches {
    ModuleSwitches::resolve(&text.parse::<DocumentMut>().unwrap()).unwrap()
}

fn fields(text: &str) -> Vec<String> {
    ModuleSwitches::errors(&text.parse::<DocumentMut>().unwrap())
        .into_iter()
        .map(|error| error.field)
        .collect()
}

#[test]
fn webhooks_require_automations() {
    assert_eq!(Module::Webhooks.requires(), &[Module::Automations]);
}

#[test]
fn dns_requires_the_proxy() {
    assert_eq!(Module::Dns.requires(), &[Module::Proxy]);
}

#[test]
fn every_name_round_trips() {
    for module in Module::ALL {
        assert_eq!(Module::from_name(module.name()), Some(module));
    }
    assert_eq!(Module::from_name("telegram"), None);
}

#[test]
fn without_any_switch_automations_and_webhooks_are_on_and_the_rest_off() {
    let resolved = switches("");
    assert!(!resolved.is_on(Module::Proxy));
    assert!(!resolved.is_on(Module::Dns));
    assert!(resolved.is_on(Module::Automations));
    assert!(resolved.is_on(Module::Webhooks));
}

#[test]
fn a_legacy_enabled_key_wins_over_the_default() {
    let resolved = switches("[proxy]\nenabled = true\n\n[dns]\nenabled = true\n");
    assert!(resolved.is_on(Module::Proxy));
    assert!(resolved.is_on(Module::Dns));
}

#[test]
fn the_modules_key_wins_over_the_legacy_key() {
    let resolved = switches("[modules]\nproxy = false\n\n[proxy]\nenabled = true\n");
    assert!(!resolved.is_on(Module::Proxy));
}

#[test]
fn an_inline_modules_table_is_read_as_well() {
    let resolved = switches("modules = { automations = false, webhooks = false }\n");
    assert!(!resolved.is_on(Module::Automations));
    assert!(!resolved.is_on(Module::Webhooks));
}

#[test]
fn an_unknown_module_or_a_value_that_is_not_a_switch_is_refused_by_its_key() {
    assert_eq!(
        fields("[modules]\ntelegram = true\n"),
        vec!["modules.telegram"]
    );
    assert_eq!(
        fields("[modules]\nproxy = \"yes\"\n"),
        vec!["modules.proxy"]
    );
    assert_eq!(fields("modules = true\n"), vec!["modules"]);
}

#[test]
fn dns_without_the_proxy_is_refused_as_modules_dns() {
    assert_eq!(fields("[dns]\nenabled = true\n"), vec!["modules.dns"]);
    let message = ModuleSwitches::errors(&"[dns]\nenabled = true\n".parse().unwrap())
        .remove(0)
        .message;
    assert!(message.contains("proxy"), "{message}");
}

#[test]
fn webhooks_without_automations_are_refused_as_modules_webhooks() {
    assert_eq!(
        fields("[modules]\nautomations = false\n"),
        vec!["modules.webhooks"]
    );
    assert!(fields("[modules]\nautomations = false\nwebhooks = false\n").is_empty());
}

#[test]
fn required_by_lists_only_enabled_modules() {
    let resolved = switches("[modules]\nwebhooks = false\n");
    assert!(resolved.required_by(Module::Automations).is_empty());
    let resolved = switches("");
    assert_eq!(
        resolved.required_by(Module::Automations),
        vec![Module::Webhooks]
    );
}
