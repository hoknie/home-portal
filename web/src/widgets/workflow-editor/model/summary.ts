import { type Step, summary as conditionSummary } from "@/entities/workflow";

export type Summary = { key: string; params: Record<string, string | number> } | { text: string };

export function summaryOf(
  step: Step,
  workflows: { id: string; title: string }[],
  automations: { id: string; title: string }[] = [],
  word: (operator: string) => string = (operator) => operator,
): Summary {
  switch (step.kind) {
    case "http":
      return { text: `${step.method ?? "GET"} ${step.url ?? ""}`.trim() };
    case "wait":
      return { key: "summaries.wait", params: { seconds: step.seconds ?? 0 } };
    case "transform":
      return { text: [step.input ?? "", ...(step.operations ?? []).map((operation) => operation.op)].filter(Boolean).join(" → ") };
    case "notify":
      return { text: step.title ? `${step.title}: ${step.text ?? ""}` : (step.text ?? "") };
    case "probe":
    case "status":
      return { text: step.service ?? "" };
    case "set":
      return { text: `${step.variable ?? ""} = ${step.json ?? step.value ?? ""}` };
    case "script":
      return { text: [step.script ?? "", ...(step.args ?? [])].join(" ").trim() };
    case "stop":
      return { key: step.outcome === "failed" ? "summaries.stopFailed" : "summaries.stopSucceeded", params: { reason: step.reason ?? "" } };
    case "automation": {
      const title = automations.find((automation) => automation.id === step.automation)?.title ?? step.automation ?? "";
      return step.wait ? { key: "summaries.automationWaited", params: { title } } : { text: title };
    }
    case "workflow":
      return { text: workflows.find((workflow) => workflow.id === step.workflow)?.title ?? step.workflow ?? "" };
    case "if":
      return { text: conditionSummary(step.condition, word) };
    case "loop":
      if (step.for_each !== undefined) {
        return { key: "summaries.forEach", params: { list: step.for_each } };
      }
      if (step.while !== undefined) {
        return { key: "summaries.while", params: { condition: conditionSummary(step.while, word) } };
      }
      return { key: "summaries.repeat", params: { count: step.repeat ?? 1 } };
    case "nothing":
      return { key: "summaries.nothing", params: {} };
    case "parallel":
      return { key: "summaries.parallel", params: { count: step.branches?.length ?? 0 } };
    default:
      return { text: "" };
  }
}
