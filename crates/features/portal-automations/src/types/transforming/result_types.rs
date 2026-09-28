use super::ValueType;

pub fn result_type(kind: &str, field: &str) -> ValueType {
    match (kind, field) {
        ("http", "status")
        | ("script", "exit_code")
        | ("probe", "latency_milliseconds")
        | ("loop", "iterations") => ValueType::Number,
        ("http", "body")
        | ("script", "stdout" | "stderr")
        | ("probe", "state" | "diagnosis")
        | ("status", "state" | "since")
        | ("if", "branch") => ValueType::Text,
        ("http", "headers") | ("workflow", "vars") => ValueType::Object,
        _ => ValueType::Any,
    }
}
