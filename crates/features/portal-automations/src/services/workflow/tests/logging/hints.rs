use super::super::running::run;
use crate::types::{Ending, StepOutcome};

#[tokio::test]
async fn a_name_where_an_id_is_needed_is_explained_with_the_id_to_use() {
    let outcome = run("[[workflows]]\nid = \"oshof\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"services\"\nkind = \"set\"\nvariable = \"svc\"\nlist = [\"{{portal.services.media.name}}\", \"{{portal.services.nas.name}}\"]\n[[workflows.steps]]\nid = \"each\"\nkind = \"loop\"\nfor_each = \"{{vars.svc}}\"\nbody = [{ id = \"status\", kind = \"status\", service = \"{{loop.item}}\" }]\n").await;
    assert_eq!(outcome.ending, Ending::Failed("no service Media".into()));
    let entry = outcome
        .trace
        .entries
        .iter()
        .find(|entry| entry.step == "status")
        .unwrap();
    assert_eq!(
        (entry.outcome, entry.iteration),
        (StepOutcome::Failed, Some(0))
    );
    assert_eq!(entry.log.values[0].template, "{{loop.item}}");
    assert_eq!(entry.log.values[0].value, "\"Media\"");
    assert_eq!(
        entry.log.lines,
        vec![
            "\"Media\" is the name of the service media; use {{portal.services.media.id}}",
            "known services: media, nas"
        ]
    );
}

#[tokio::test]
async fn an_unknown_id_that_is_no_name_lists_the_known_ids_only() {
    let outcome = run("[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"try\"\nkind = \"probe\"\nservice = \"ghost\"\n").await;
    assert_eq!(
        outcome.trace.entries[0].log.lines,
        vec!["known services: media, nas"]
    );
}
