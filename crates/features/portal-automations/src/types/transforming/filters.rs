use super::ArgumentDescription as Argument;
use super::ArgumentType::{Any as AnyArgument, Number as NumberArgument, Text as TextArgument};
use super::FilterDescription as Filter;
use super::ValueType::{Any, Boolean, List, Number, Object, Text};

const NONE: &[Argument] = &[];
const TEXT: &[super::ValueType] = &[Text];
const NUMBER: &[super::ValueType] = &[Number];
const LIST: &[super::ValueType] = &[List];
const OBJECT: &[super::ValueType] = &[Object];
const ANY: &[super::ValueType] = &[Any];

const fn filter(
    name: &'static str,
    accepts: &'static [super::ValueType],
    gives: super::ValueType,
    arguments: &'static [Argument],
) -> Filter {
    Filter {
        name,
        accepts,
        gives,
        element: false,
        arguments,
    }
}

const fn element(filter: Filter) -> Filter {
    Filter {
        element: true,
        ..filter
    }
}

pub const FILTERS: &[Filter] = &[
    element(filter("upper", TEXT, Text, NONE)),
    element(filter("lower", TEXT, Text, NONE)),
    element(filter("trim", TEXT, Text, NONE)),
    element(filter(
        "replace",
        TEXT,
        Text,
        &[
            Argument::required("from", TextArgument),
            Argument::required("to", TextArgument),
        ],
    )),
    filter(
        "split",
        TEXT,
        List,
        &[Argument::required("separator", TextArgument)],
    ),
    filter(
        "slice",
        &[Text, List],
        Any,
        &[
            Argument::required("start", NumberArgument),
            Argument::optional("end", NumberArgument),
        ],
    ),
    element(filter(
        "starts_with",
        TEXT,
        Boolean,
        &[Argument::required("text", TextArgument)],
    )),
    filter(
        "contains",
        &[Text, List],
        Boolean,
        &[Argument::required("value", AnyArgument)],
    ),
    filter("length", &[Text, List, Object], Number, NONE),
    filter(
        "default",
        ANY,
        Any,
        &[Argument::required("value", AnyArgument)],
    ),
    element(filter("number", &[Text, Number], Number, NONE)),
    element(filter(
        "round",
        NUMBER,
        Number,
        &[Argument::optional("digits", NumberArgument)],
    )),
    element(filter("floor", NUMBER, Number, NONE)),
    element(filter("ceil", NUMBER, Number, NONE)),
    element(filter("abs", NUMBER, Number, NONE)),
    filter("first", LIST, Any, NONE),
    filter("last", LIST, Any, NONE),
    filter(
        "join",
        LIST,
        Text,
        &[Argument::required("separator", TextArgument)],
    ),
    filter("sort", LIST, List, NONE),
    filter("reverse", LIST, List, NONE),
    filter("unique", LIST, List, NONE),
    filter(
        "pluck",
        LIST,
        List,
        &[Argument::required("key", TextArgument)],
    ),
    filter("sum", LIST, Number, NONE),
    filter("min", LIST, Number, NONE),
    filter("max", LIST, Number, NONE),
    filter("keys", OBJECT, List, NONE),
    filter("values", OBJECT, List, NONE),
    filter(
        "get",
        &[Object, List],
        Any,
        &[Argument::required("key", TextArgument)],
    ),
    filter("json", ANY, Text, NONE),
    filter("parse", TEXT, Any, NONE),
];

pub fn filter_named(name: &str) -> Option<&'static Filter> {
    FILTERS.iter().find(|filter| filter.name == name)
}
