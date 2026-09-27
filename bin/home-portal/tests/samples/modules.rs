use portal_modules::{ModuleResponse, ModulesResponse};

use crate::check;

fn module(
    name: &'static str,
    enabled: bool,
    requires: Vec<&'static str>,
    required_by: Vec<&'static str>,
) -> ModuleResponse {
    ModuleResponse {
        name,
        enabled,
        requires,
        required_by,
    }
}

#[test]
fn the_modules_sample_matches_its_serializer() {
    let modules = ModulesResponse {
        modules: vec![
            module("proxy", true, vec![], vec!["dns"]),
            module("dns", true, vec!["proxy"], vec![]),
            module("automations", true, vec![], vec![]),
            module("webhooks", false, vec!["automations"], vec![]),
        ],
    };
    check("modules", serde_json::to_value(modules).unwrap());
}
