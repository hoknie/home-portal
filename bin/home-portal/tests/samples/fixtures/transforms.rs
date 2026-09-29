use portal_automations::{RawOperation, TransformValue};
use serde_json::{Value, json};

use crate::check;

fn cases() -> Vec<(&'static str, Value, Value)> {
    let disks = json!([
        {"name": "sda", "health": "ok", "size": 2},
        {"name": "sdb", "health": "failing", "size": 3.5},
        {"name": "sdc", "health": "failing", "size": 1}
    ]);
    vec![
        ("upper", json!("nas"), json!([{"op": "upper"}])),
        (
            "get with a key from the value itself",
            json!({"which": "nas", "nas": 2, "media": 1}),
            json!([{"op": "get", "args": ["{{vars.input.which}}"]}]),
        ),
        (
            "sort_by with a templated key",
            json!([{"k": "b", "n": 2}, {"k": "a", "n": 1}]),
            json!([{"op": "sort_by", "key": "{{vars.input.0 | keys | first}}"}]),
        ),
        (
            "map reading a key named by the item",
            json!([{"name": "a", "a": 1}, {"name": "b", "b": 2}]),
            json!([{"op": "map", "to": "{{item | get(item.name)}}"}]),
        ),
        ("lower", json!("NAS"), json!([{"op": "lower"}])),
        ("trim", json!("  nas "), json!([{"op": "trim"}])),
        (
            "replace",
            json!("a-b-c"),
            json!([{"op": "replace", "args": ["-", "+"]}]),
        ),
        (
            "split",
            json!("a,b,,c"),
            json!([{"op": "split", "args": [","]}]),
        ),
        (
            "slice text",
            json!("homeportal"),
            json!([{"op": "slice", "args": [4]}]),
        ),
        (
            "slice from the end",
            json!([1, 2, 3, 4]),
            json!([{"op": "slice", "args": [-3, -1]}]),
        ),
        (
            "starts_with",
            json!("portal"),
            json!([{"op": "starts_with", "args": ["por"]}]),
        ),
        (
            "contains text",
            json!("portal"),
            json!([{"op": "contains", "args": ["tal"]}]),
        ),
        (
            "contains in a list",
            json!([1, "2", 3]),
            json!([{"op": "contains", "args": [2]}]),
        ),
        ("length of text", json!("héllo"), json!([{"op": "length"}])),
        (
            "length of an object",
            json!({"a": 1, "b": 2}),
            json!([{"op": "length"}]),
        ),
        (
            "default of empty text",
            json!(""),
            json!([{"op": "default", "args": ["none"]}]),
        ),
        (
            "default keeps a value",
            json!(0),
            json!([{"op": "default", "args": ["none"]}]),
        ),
        ("number", json!(" 42.50 "), json!([{"op": "number"}])),
        (
            "number refuses text",
            json!("abc"),
            json!([{"op": "number"}]),
        ),
        (
            "round",
            json!(7.12645),
            json!([{"op": "round", "args": [2]}]),
        ),
        ("round to whole", json!(2.5), json!([{"op": "round"}])),
        (
            "floor, ceil and abs",
            json!(-2.5),
            json!([{"op": "floor"}, {"op": "abs"}, {"op": "ceil"}]),
        ),
        (
            "first and last",
            json!([[1, 2], [3]]),
            json!([{"op": "last"}, {"op": "first"}]),
        ),
        ("first of nothing", json!([]), json!([{"op": "first"}])),
        (
            "join",
            json!(["a", 1, true, null]),
            json!([{"op": "join", "args": ["-"]}]),
        ),
        (
            "sort mixed",
            json!([3, "b", 1, "a", null, 2.5]),
            json!([{"op": "sort"}]),
        ),
        ("reverse", json!([1, 2, 3]), json!([{"op": "reverse"}])),
        (
            "unique",
            json!([1, 2, 1, "1", 2]),
            json!([{"op": "unique"}]),
        ),
        (
            "pluck with a path",
            json!([{"a": {"b": 1}}, {"a": {}}]),
            json!([{"op": "pluck", "args": ["a.b"]}]),
        ),
        (
            "sum, min and max",
            json!([4, 1.5, 9]),
            json!([{"op": "sum"}]),
        ),
        ("min", json!([4, 1.5, 9]), json!([{"op": "min"}])),
        ("max of nothing", json!([]), json!([{"op": "max"}])),
        ("sum refuses text", json!([1, "2"]), json!([{"op": "sum"}])),
        (
            "keys and values",
            json!({"b": 1, "a": 2}),
            json!([{"op": "keys"}]),
        ),
        ("values", json!({"b": 1, "a": 2}), json!([{"op": "values"}])),
        (
            "get",
            json!({"a": [10, 20]}),
            json!([{"op": "get", "args": ["a.1"]}]),
        ),
        ("json", json!({"a": [1, "x"]}), json!([{"op": "json"}])),
        (
            "parse",
            json!("{\"a\": [1, 2]}"),
            json!([{"op": "parse"}, {"op": "get", "args": ["a"]}]),
        ),
        ("parse refuses", json!("{nope"), json!([{"op": "parse"}])),
        (
            "a filter on the wrong type",
            json!(200),
            json!([{"op": "join", "args": [", "]}]),
        ),
        (
            "nothing passes through",
            Value::Null,
            json!([{"op": "upper"}, {"op": "length"}]),
        ),
        (
            "unhealthy disks",
            disks.clone(),
            json!([
                {"op": "filter", "where": {"left": "{{item.health}}", "op": "!=", "right": "ok"}},
                {"op": "pluck", "args": ["name"]},
                {"op": "join", "args": [", "]}
            ]),
        ),
        (
            "filter with a group",
            disks.clone(),
            json!([
                {"op": "filter", "where": {"any": [{"left": "{{item.size}}", "op": ">", "right": "3"}, {"left": "{{index}}", "op": "==", "right": "0"}]}},
                {"op": "pluck", "args": ["name"]}
            ]),
        ),
        (
            "map with filters",
            disks.clone(),
            json!([
                {"op": "map", "to": "{{index}}. {{item.name | upper}} ({{item.size | round}})"}
            ]),
        ),
        (
            "map keeps values",
            disks.clone(),
            json!([{"op": "map", "to": "{{item.size}}"}, {"op": "sum"}]),
        ),
        (
            "sort_by descending",
            disks.clone(),
            json!([
                {"op": "sort_by", "key": "size", "order": "desc"},
                {"op": "pluck", "args": ["name"]}
            ]),
        ),
        (
            "group_by",
            disks.clone(),
            json!([{"op": "group_by", "key": "health"}, {"op": "keys"}]),
        ),
        (
            "count_by",
            json!([{"state": "up"}, {"state": "down"}, {"state": "up"}]),
            json!([{"op": "count_by", "key": "state"}]),
        ),
        (
            "a list operation on text",
            json!("sda"),
            json!([{"op": "count_by", "key": "state"}]),
        ),
        (
            "upper over a split text",
            json!("nas,router"),
            json!([{"op": "split", "args": [","]}, {"op": "upper"}]),
        ),
        (
            "round and abs over a list",
            json!([1.26, null, -2]),
            json!([{"op": "round", "args": [1]}, {"op": "abs"}]),
        ),
        (
            "upper over a mixed list",
            json!(["nas", 3]),
            json!([{"op": "upper"}]),
        ),
        (
            "length of a list stays whole",
            json!(["a", "b"]),
            json!([{"op": "length"}]),
        ),
        (
            "each through a chain",
            json!([" nas ", " router "]),
            json!([
                {"op": "each", "operations": [{"op": "trim"}, {"op": "upper"}, {"op": "replace", "args": ["R", "r"]}]}
            ]),
        ),
        (
            "each with map and join",
            json!([[1, 2], [3]]),
            json!([
                {"op": "each", "operations": [{"op": "map", "to": "{{item}}{{index}}"}, {"op": "join", "args": ["-"]}]}
            ]),
        ),
        (
            "each with a nested filter",
            json!([[1, 5], [7, 2]]),
            json!([
                {"op": "each", "operations": [{"op": "filter", "where": {"left": "{{item}}", "op": ">", "right": "3"}}, {"op": "length"}]}
            ]),
        ),
        (
            "a failing nested operation",
            json!(["1", "x"]),
            json!([{"op": "each", "operations": [{"op": "number"}]}]),
        ),
    ]
}

#[test]
fn transform_fixtures_are_shared_with_the_interface() {
    let transform = TransformValue;
    let cases: Vec<Value> = cases()
        .into_iter()
        .map(|(name, input, operations)| {
            let parsed: Vec<RawOperation> = serde_json::from_value(operations.clone()).unwrap();
            let outcome = match transform.run(input.clone(), &parsed) {
                Ok(value) => json!({"value": value}),
                Err(error) => json!({"error": error}),
            };
            json!({"name": name, "input": input, "operations": operations, "outcome": outcome})
        })
        .collect();
    check("transforms", json!({ "cases": cases }));
}
