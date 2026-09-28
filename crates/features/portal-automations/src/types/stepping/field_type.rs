#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Template,
    TemplateList,
    TemplateTable,
    Integer,
    Boolean,
    Choice(&'static [&'static str]),
    Condition,
    Steps,
    Branches,
    Name,
    Workflow,
    Script,
    Sample,
    Channel,
    Operations,
    Automation,
}

impl FieldType {
    pub fn name(self) -> &'static str {
        match self {
            FieldType::Template => "template",
            FieldType::TemplateList => "template-list",
            FieldType::TemplateTable => "template-table",
            FieldType::Integer => "integer",
            FieldType::Boolean => "boolean",
            FieldType::Choice(_) => "choice",
            FieldType::Condition => "condition",
            FieldType::Steps => "steps",
            FieldType::Branches => "branches",
            FieldType::Name => "name",
            FieldType::Workflow => "workflow",
            FieldType::Script => "script",
            FieldType::Sample => "sample",
            FieldType::Channel => "channel",
            FieldType::Operations => "operations",
            FieldType::Automation => "automation",
        }
    }

    pub fn choices(self) -> &'static [&'static str] {
        match self {
            FieldType::Choice(choices) => choices,
            _ => &[],
        }
    }
}
