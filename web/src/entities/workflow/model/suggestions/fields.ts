import type { Condition, Step } from "../schema";

export type TemplateField = { field: string; text: string };

function conditionFields(condition: Condition | undefined, path: string, found: TemplateField[]) {
  if (condition === undefined) {
    return;
  }
  const rows = condition.all ?? condition.any;
  if (rows !== undefined) {
    const key = condition.all !== undefined ? "all" : "any";
    rows.forEach((row, index) => conditionFields(row, `${path}.${key}[${index}]`, found));
    return;
  }
  found.push({ field: `${path}.left`, text: condition.left ?? "" });
  if (condition.right !== undefined) {
    found.push({ field: `${path}.right`, text: condition.right });
  }
}

export function templatesOf(step: Step): TemplateField[] {
  const found: TemplateField[] = [];
  const text = (field: keyof Step) => {
    const value = step[field];
    if (typeof value === "string") {
      found.push({ field, text: value });
    }
  };
  switch (step.kind) {
    case "if":
      conditionFields(step.condition, "condition", found);
      break;
    case "loop":
      text("for_each");
      conditionFields(step.while, "while", found);
      break;
    case "workflow":
      for (const [name, value] of Object.entries(step.inputs ?? {})) {
        found.push({ field: `inputs.${name}`, text: value });
      }
      break;
    case "stop":
      text("reason");
      break;
    case "set":
      text("value");
      text("json");
      (step.list ?? []).forEach((item, index) => found.push({ field: `list[${index}]`, text: item }));
      for (const [key, item] of Object.entries(step.object ?? {})) {
        found.push({ field: `object.${key}`, text: item });
      }
      break;
    case "http":
      text("url");
      for (const [name, value] of Object.entries(step.headers ?? {})) {
        found.push({ field: `headers.${name}`, text: value });
      }
      if (typeof step.body === "string") {
        found.push({ field: "body", text: step.body });
      }
      break;
    case "script":
      (step.args ?? []).forEach((argument, index) => found.push({ field: `args[${index}]`, text: argument }));
      break;
    case "transform":
      text("input");
      (step.operations ?? []).forEach((operation, index) => {
        if (operation.op === "filter") {
          conditionFields(operation.where, `operations[${index}].where`, found);
        } else if (operation.op === "map") {
          found.push({ field: `operations[${index}].to`, text: operation.to ?? "" });
        }
      });
      break;
    case "notify":
      text("title");
      text("text");
      break;
    case "probe":
    case "status":
      text("service");
      break;
    default:
      break;
  }
  return found;
}
