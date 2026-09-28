use serde_json::json;

use super::running::run;
use super::support::fields;
use crate::services::transform_sample;
use crate::types::RawOperation;

fn operations(value: serde_json::Value) -> Vec<RawOperation> {
    serde_json::from_value(value).unwrap()
}

const DISKS: &str = "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"list\"\nkind = \"set\"\nvariable = \"disks\"\njson = '[{\"name\": \"sda\", \"health\": \"ok\"}, {\"name\": \"sdb\", \"health\": \"failing\"}, {\"name\": \"sdc\", \"health\": \"failing\"}]'\n";

#[tokio::test]
async fn unhealthy_disks_are_kept_named_and_joined() {
    let outcome = run(&format!("{DISKS}[[workflows.steps]]\nid = \"bad\"\nkind = \"transform\"\ninput = \"{{{{vars.disks}}}}\"\noperations = [{{ op = \"filter\", where = {{ left = \"{{{{item.health}}}}\", op = \"!=\", right = \"ok\" }} }}, {{ op = \"pluck\", args = [\"name\"] }}, {{ op = \"join\", args = [\", \"] }}]\n")).await;
    assert_eq!(outcome.frame.steps["bad"], json!({"value": "sdb, sdc"}));
    let entry = &outcome.trace.entries[1];
    assert_eq!(entry.detail, "filter → pluck → join");
    let snapshots: serde_json::Value =
        serde_json::from_str(entry.output.as_deref().unwrap()).unwrap();
    assert_eq!(snapshots[1], json!(["sdb", "sdc"]));
}

#[test]
fn counting_by_state_gives_an_object_of_counts() {
    let value = transform_sample(
        json!([{"state": "up"}, {"state": "down"}, {"state": "up"}]),
        &operations(json!([{"op": "count_by", "key": "state"}])),
    );
    assert_eq!(value, Ok(json!({"up": 2, "down": 1})));
}

#[test]
fn map_sort_and_group_see_the_item_and_its_index() {
    let items = json!([{"n": "b", "size": 2}, {"n": "a", "size": 9}, {"n": "c", "size": 2}]);
    assert_eq!(
        transform_sample(
            items.clone(),
            &operations(json!([
                {"op": "sort_by", "key": "size", "order": "desc"},
                {"op": "map", "to": "{{index}}:{{item.n | upper}}"}
            ]))
        ),
        Ok(json!(["0:A", "1:B", "2:C"]))
    );
    assert_eq!(
        transform_sample(
            items,
            &operations(json!([{"op": "group_by", "key": "size"}]))
        ),
        Ok(
            json!({"2": [{"n": "b", "size": 2}, {"n": "c", "size": 2}], "9": [{"n": "a", "size": 9}]})
        )
    );
}

#[test]
fn a_type_mismatch_names_the_operation_position() {
    assert_eq!(
        transform_sample(
            json!(7),
            &operations(json!([{"op": "abs"}, {"op": "join", "args": [","]}]))
        ),
        Err("operation 2 (join): join takes a list and got a number".to_string())
    );
}

#[test]
fn operations_are_checked_at_load_by_name_arguments_and_scope() {
    let step = |operations: &str| {
        fields(&format!(
            "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"t\"\nkind = \"transform\"\ninput = \"[]\"\noperations = {operations}\n"
        ))
    };
    assert_eq!(step("[]"), vec!["workflows[0].steps[0].operations"]);
    assert_eq!(
        step(
            "[{ op = \"shout\" }, { op = \"join\" }, { op = \"sort_by\", key = \"a\", order = \"up\" }, { op = \"map\", to = \"{{loop.item}}\" }]"
        ),
        vec![
            "workflows[0].steps[0].operations[0].op",
            "workflows[0].steps[0].operations[1].args",
            "workflows[0].steps[0].operations[2].order",
        ]
    );
    assert_eq!(
        step("[{ op = \"map\", to = \"{{loop.item}}\" }]"),
        vec!["workflows[0].steps[0].operations[0].to"]
    );
    let many = vec!["{ op = \"reverse\" }"; 21].join(", ");
    assert_eq!(
        step(&format!("[{many}]")),
        vec!["workflows[0].steps[0].operations"]
    );
}

#[test]
fn element_filters_apply_to_each_item_of_a_list() {
    assert_eq!(
        transform_sample(
            json!("nas,router"),
            &operations(json!([{"op": "split", "args": [","]}, {"op": "upper"}]))
        ),
        Ok(json!(["NAS", "ROUTER"]))
    );
    assert_eq!(
        transform_sample(
            json!([1.26, null, -2]),
            &operations(json!([{"op": "round", "args": [1]}, {"op": "abs"}]))
        ),
        Ok(json!([1.3, null, 2]))
    );
    assert_eq!(
        transform_sample(json!(["nas", 3]), &operations(json!([{"op": "upper"}]))),
        Err("operation 1 (upper): upper takes text and got a number at position 1".to_string())
    );
    assert_eq!(
        transform_sample(json!(["a", "b"]), &operations(json!([{"op": "length"}]))),
        Ok(json!(2))
    );
}

#[test]
fn each_runs_a_nested_chain_on_every_element_and_names_a_nested_failure() {
    assert_eq!(
        transform_sample(
            json!([" nas ", " router "]),
            &operations(
                json!([{"op": "each", "operations": [{"op": "trim"}, {"op": "upper"}, {"op": "replace", "args": ["R", "r"]}]}])
            )
        ),
        Ok(json!(["NAS", "rOUTEr"]))
    );
    assert_eq!(
        transform_sample(
            json!(["1", "x"]),
            &operations(json!([{"op": "each", "operations": [{"op": "number"}]}]))
        ),
        Err("operation 1.1 (number): number cannot read \"x\" as a number".to_string())
    );
    assert_eq!(
        transform_sample(
            json!([[1, 2], [3]]),
            &operations(
                json!([{"op": "each", "operations": [{"op": "map", "to": "{{item}}{{index}}"}, {"op": "join", "args": ["-"]}]}])
            )
        ),
        Ok(json!(["10-21", "30"]))
    );
}

#[test]
fn each_is_checked_at_load_for_its_depth_and_its_nested_operations() {
    let step = |operations: &str| {
        fields(&format!(
            "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"t\"\nkind = \"transform\"\ninput = \"[]\"\noperations = {operations}\n"
        ))
    };
    assert_eq!(
        step(
            "[{ op = \"each\", operations = [{ op = \"shout\" }] }, { op = \"each\", operations = [] }]"
        ),
        vec![
            "workflows[0].steps[0].operations[0].operations[0].op",
            "workflows[0].steps[0].operations[1].operations",
        ]
    );
    let deep = "{ op = \"each\", operations = [{ op = \"each\", operations = [{ op = \"each\", operations = [{ op = \"each\", operations = [{ op = \"trim\" }] }] }] }] }";
    assert_eq!(
        step(&format!("[{deep}]")),
        vec!["workflows[0].steps[0].operations[0].operations[0].operations[0].operations[0]"]
    );
    assert!(
        step("[{ op = \"each\", operations = [{ op = \"map\", to = \"{{item | upper}}\" }] }]")
            .is_empty()
    );
}
