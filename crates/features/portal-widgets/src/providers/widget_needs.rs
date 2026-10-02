use portal_feature::{Action, Area, NeedScope, Right, WidgetNeed};
use serde_json::Value;

pub const RUN_AUTOMATIONS: Right = Right::new(Area::Automations, Action::Execute);
pub const RUN_WORKFLOWS: Right = Right::new(Area::Workflows, Action::Execute);

pub fn needs_of(settings: &Value) -> Vec<WidgetNeed> {
    let mut needs = Vec::new();
    if let Some(source) = settings.get("source")
        && let Some(need) = source_need(source)
    {
        needs.push(WidgetNeed::new(
            "source",
            need.0,
            need.1,
            NeedScope::SameWidget,
        ));
    }
    buttons(settings.get("blocks"), "blocks", &mut needs);
    needs
}

pub fn source_need(source: &Value) -> Option<(Right, String)> {
    if let Some(workflow) = source.get("workflow").and_then(Value::as_str) {
        return Some((
            RUN_WORKFLOWS,
            format!(
                "workflow:{workflow}:{}",
                source.get("inputs").cloned().unwrap_or(Value::Null)
            ),
        ));
    }
    source.get("script").and_then(Value::as_str).map(|script| {
        (
            RUN_AUTOMATIONS,
            format!(
                "script:{script}:{}",
                source.get("args").cloned().unwrap_or(Value::Null)
            ),
        )
    })
}

pub fn action_need(action: &Value) -> Option<(Right, String)> {
    if let Some(automation) = action.get("automation").and_then(Value::as_str) {
        return Some((
            RUN_AUTOMATIONS,
            format!(
                "automation:{automation}:{}",
                action.get("fields").cloned().unwrap_or(Value::Null)
            ),
        ));
    }
    action
        .get("workflow")
        .and_then(Value::as_str)
        .map(|workflow| {
            (
                RUN_WORKFLOWS,
                format!(
                    "workflow:{workflow}:{}",
                    action.get("inputs").cloned().unwrap_or(Value::Null)
                ),
            )
        })
}

fn buttons(blocks: Option<&Value>, path: &str, needs: &mut Vec<WidgetNeed>) {
    let Some(list) = blocks.and_then(Value::as_array) else {
        return;
    };
    for (index, block) in list.iter().enumerate() {
        let here = format!("{path}[{index}]");
        if let Some(action) = block.get("action")
            && let Some((right, fingerprint)) = action_need(action)
        {
            needs.push(WidgetNeed::new(
                format!("{here}.action"),
                right,
                fingerprint,
                NeedScope::SameWidget,
            ));
        }
        buttons(block.get("blocks"), &format!("{here}.blocks"), needs);
    }
}
