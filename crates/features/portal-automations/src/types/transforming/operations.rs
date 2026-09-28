use super::ArgumentDescription as Argument;
use super::ArgumentType::{Condition, Operations, Order, Template, Text};
use super::FilterDescription as Description;
use super::ValueType::{List, Object};

pub const MOST_OPERATIONS: usize = 20;
pub const DEEPEST_EACH: usize = 3;

pub const OPERATIONS: &[Description] = &[
    Description {
        name: "filter",
        accepts: &[List],
        gives: List,
        element: false,
        arguments: &[Argument::required("where", Condition)],
    },
    Description {
        name: "map",
        accepts: &[List],
        gives: List,
        element: false,
        arguments: &[Argument::required("to", Template)],
    },
    Description {
        name: "sort_by",
        accepts: &[List],
        gives: List,
        element: false,
        arguments: &[
            Argument::required("key", Text),
            Argument::optional("order", Order),
        ],
    },
    Description {
        name: "group_by",
        accepts: &[List],
        gives: Object,
        element: false,
        arguments: &[Argument::required("key", Text)],
    },
    Description {
        name: "count_by",
        accepts: &[List],
        gives: Object,
        element: false,
        arguments: &[Argument::required("key", Text)],
    },
    Description {
        name: "each",
        accepts: &[List],
        gives: List,
        element: false,
        arguments: &[Argument::required("operations", Operations)],
    },
];
