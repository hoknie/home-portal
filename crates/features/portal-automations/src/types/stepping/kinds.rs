use super::FieldType::{
    Automation, Boolean, Branches, Channel, Choice, Condition, Integer, Name, Operations, Sample,
    Script, Steps, Template, TemplateList, TemplateTable, Workflow,
};
use super::{FieldDescription as Field, KindDescription as Kind, KindGroup};
use crate::types::MOST_OPERATIONS;

pub const OUTCOMES: &[&str] = &["succeeded", "failed"];
pub const METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"];

pub const KINDS: &[Kind] = &[
    Kind {
        name: "if",
        group: KindGroup::Flow,
        fields: &[
            Field::of("condition", Condition).required(),
            Field::of("then", Steps).required(),
            Field::of("else", Steps),
        ],
        results: &["branch"],
    },
    Kind {
        name: "loop",
        group: KindGroup::Flow,
        fields: &[
            Field::of("repeat", Integer).between(1, 100),
            Field::of("for_each", Template),
            Field::of("while", Condition),
            Field::of("max_iterations", Integer)
                .between(1, 100)
                .defaulting("100"),
            Field::of("body", Steps).required(),
        ],
        results: &["iterations"],
    },
    Kind {
        name: "parallel",
        group: KindGroup::Flow,
        fields: &[Field::of("branches", Branches).required().between(2, 4)],
        results: &[],
    },
    Kind {
        name: "workflow",
        group: KindGroup::Flow,
        fields: &[
            Field::of("workflow", Workflow).required(),
            Field::of("inputs", TemplateTable),
        ],
        results: &["vars"],
    },
    Kind {
        name: "nothing",
        group: KindGroup::Flow,
        fields: &[],
        results: &[],
    },
    Kind {
        name: "stop",
        group: KindGroup::Flow,
        fields: &[
            Field::of("outcome", Choice(OUTCOMES)).required(),
            Field::of("reason", Template),
        ],
        results: &[],
    },
    Kind {
        name: "set",
        group: KindGroup::Data,
        fields: &[
            Field::of("variable", Name).required(),
            Field::of("value", Template),
            Field::of("json", Template),
            Field::of("list", TemplateList),
            Field::of("object", TemplateTable),
        ],
        results: &["value"],
    },
    Kind {
        name: "wait",
        group: KindGroup::Data,
        fields: &[Field::of("seconds", Integer).required().between(1, 3600)],
        results: &[],
    },
    Kind {
        name: "transform",
        group: KindGroup::Data,
        fields: &[
            Field::of("input", Template).required(),
            Field::of("operations", Operations)
                .required()
                .between(1, MOST_OPERATIONS as i64),
        ],
        results: &["value"],
    },
    Kind {
        name: "http",
        group: KindGroup::Actions,
        fields: &[
            Field::of("method", Choice(METHODS)).defaulting("GET"),
            Field::of("url", Template).required(),
            Field::of("headers", TemplateTable),
            Field::of("body", Template),
            Field::of("timeout_seconds", Integer)
                .between(1, 60)
                .defaulting("10"),
            Field::of("fail_on_error", Boolean).defaulting("true"),
            Field::of("response_sample", Sample),
        ],
        results: &["status", "headers", "body", "json"],
    },
    Kind {
        name: "script",
        group: KindGroup::Actions,
        fields: &[
            Field::of("script", Script).required(),
            Field::of("args", TemplateList),
            Field::of("env", TemplateTable),
            Field::of("stdin", Template),
            Field::of("timeout_seconds", Integer)
                .between(1, 3600)
                .defaulting("60"),
        ],
        results: &["exit_code", "stdout", "stderr"],
    },
    Kind {
        name: "notify",
        group: KindGroup::Actions,
        fields: &[
            Field::of("text", Template).required(),
            Field::of("title", Template),
            Field::of("channel", Channel),
        ],
        results: &[],
    },
    Kind {
        name: "automation",
        group: KindGroup::Actions,
        fields: &[
            Field::of("automation", Automation).required(),
            Field::of("fields", TemplateTable),
            Field::of("wait", Boolean).defaulting("false"),
        ],
        results: &["run_id", "outcome"],
    },
    Kind {
        name: "probe",
        group: KindGroup::Actions,
        fields: &[Field::of("service", Template).required()],
        results: &["state", "latency_milliseconds", "diagnosis"],
    },
    Kind {
        name: "status",
        group: KindGroup::Actions,
        fields: &[Field::of("service", Template).required()],
        results: &["state", "since"],
    },
];

pub fn kind_named(name: &str) -> Option<&'static Kind> {
    KINDS.iter().find(|kind| kind.name == name)
}
