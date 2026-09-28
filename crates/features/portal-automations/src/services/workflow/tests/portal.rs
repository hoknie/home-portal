use serde_json::json;

use super::running::run;
use super::support::fields;
use crate::types::Ending;

const HEAD: &str = "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n";

#[tokio::test]
async fn a_service_s_name_and_state_are_read_from_the_portal() {
    let outcome = run(&format!("{HEAD}[[workflows.steps]]\nid = \"tell\"\nkind = \"notify\"\ntext = \"{{{{portal.services.media.name}}}} is {{{{portal.services.media.state}}}}\"\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(*outcome.actions.sent.lock().unwrap(), vec!["Media is down"]);
}

#[tokio::test]
async fn a_module_switch_chooses_a_branch() {
    let outcome = run(&format!("{HEAD}[[workflows.steps]]\nid = \"check\"\nkind = \"if\"\ncondition = {{ left = \"{{{{portal.modules.users.is_enabled}}}}\", op = \"==\", right = \"true\" }}\nthen = [{{ id = \"on\", kind = \"nothing\" }}]\nelse = [{{ id = \"off\", kind = \"nothing\" }}]\n")).await;
    assert_eq!(outcome.frame.steps["check"], json!({"branch": "else"}));
}

#[tokio::test]
async fn a_step_after_a_probe_reads_the_new_state_and_the_network_and_lists() {
    let outcome = run(&format!("{HEAD}[[workflows.steps]]\nid = \"before\"\nkind = \"set\"\nvariable = \"before\"\nvalue = \"{{{{portal.services.nas.state}}}}\"\n[[workflows.steps]]\nid = \"look\"\nkind = \"probe\"\nservice = \"nas\"\n[[workflows.steps]]\nid = \"after\"\nkind = \"set\"\nvariable = \"after\"\nobject = {{ state = \"{{{{portal.services.nas.state}}}}\", port = \"{{{{portal.network.port}}}}\", count = \"{{{{portal.services | length}}}}\", places = \"{{{{portal.environments}}}}\" }}\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(outcome.frame.vars["before"], json!("down"));
    assert_eq!(
        outcome.frame.vars["after"],
        json!({"state": "up", "port": 8080, "count": 2, "places": ["local", "vpn"]})
    );
}

#[test]
fn unknown_services_modules_and_fields_are_refused_at_load() {
    let found = fields(&format!(
        "[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://nas.lan\"\n\n{HEAD}[[workflows.steps]]\nid = \"a\"\nkind = \"notify\"\ntext = \"{{{{portal.services.nope.name}}}}\"\n[[workflows.steps]]\nid = \"b\"\nkind = \"notify\"\ntext = \"{{{{portal.modules.coffee.is_enabled}}}}\"\n[[workflows.steps]]\nid = \"c\"\nkind = \"notify\"\ntext = \"{{{{portal.services.nas.colour}}}} {{{{portal.network.ip}}}}\"\n[[workflows.steps]]\nid = \"d\"\nkind = \"notify\"\ntext = \"{{{{portal.services.nas.name}}}} {{{{portal.network.url}}}} {{{{portal.modules.users.is_enabled}}}}\"\n"
    ));
    assert_eq!(
        found,
        vec![
            "workflows[0].steps[0].text",
            "workflows[0].steps[1].text",
            "workflows[0].steps[2].text",
            "workflows[0].steps[2].text",
        ]
    );
}

#[test]
fn a_text_filter_on_a_portal_boolean_is_refused() {
    let found = fields(&format!(
        "{HEAD}[[workflows.steps]]\nid = \"a\"\nkind = \"notify\"\ntext = \"{{{{portal.modules.users.is_enabled | upper}}}}\"\n"
    ));
    assert_eq!(found, vec!["workflows[0].steps[0].text"]);
}
