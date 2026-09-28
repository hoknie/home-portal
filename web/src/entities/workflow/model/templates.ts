import { plainInput } from "./inputs";
import type { Step, WorkflowRequest } from "./schema";

export const TEMPLATE_NAMES = ["retry", "notify-down", "parallel-check", "daily-report", "empty"] as const;

export type TemplateName = (typeof TEMPLATE_NAMES)[number];

export type WorkflowTemplate = { name: TemplateName; draft: WorkflowRequest };

function draft(id: string, title: string, inputs: string[], steps: Step[]): WorkflowRequest {
  return { id, title, enabled: true, description: null, tags: [], timeout_seconds: 300, inputs: inputs.map(plainInput), steps };
}

export const TEMPLATES: WorkflowTemplate[] = [
  {
    name: "retry",
    draft: draft("retry-request", "Retry a request", ["url"], [
      {
        id: "retry",
        label: "Try three times",
        kind: "loop",
        repeat: 3,
        body: [
          { id: "ping", label: "Ask the address", kind: "http", method: "GET", url: "{{inputs.url}}", fail_on_error: false },
          {
            id: "answered",
            label: "Did it answer?",
            kind: "if",
            condition: { left: "{{steps.ping.status}}", op: "==", right: "200" },
            then: [{ id: "done", kind: "stop", outcome: "succeeded", reason: "answered" }],
            else: [{ id: "pause", label: "Wait a little", kind: "wait", seconds: 10 }],
          },
        ],
      },
      { id: "give_up", label: "Give up", kind: "stop", outcome: "failed", reason: "no answer after three tries" },
    ]),
  },
  {
    name: "notify-down",
    draft: draft("notify-down", "Tell me when a service is down", ["service"], [
      { id: "check", label: "Check the service", kind: "probe", service: "{{inputs.service}}" },
      {
        id: "down",
        label: "Is it down?",
        kind: "if",
        condition: { left: "{{steps.check.state}}", op: "!=", right: "up" },
        then: [{ id: "tell", label: "Send a message", kind: "notify", text: "{{inputs.service}} is {{steps.check.state}}" }],
        else: [],
      },
    ]),
  },
  {
    name: "parallel-check",
    draft: draft("check-several", "Check several services at once", [], [
      {
        id: "together",
        label: "Check together",
        kind: "parallel",
        branches: [
          [{ id: "first", kind: "status", service: "nas" }],
          [{ id: "second", kind: "status", service: "router" }],
        ],
      },
      { id: "report", label: "Report", kind: "notify", text: "nas: {{steps.first.state}}, router: {{steps.second.state}}" },
    ]),
  },
  {
    name: "daily-report",
    draft: draft("daily-report", "Daily report", [], [
      { id: "greeting", label: "Start the text", kind: "set", variable: "report", value: "Good morning" },
      { id: "nas", label: "NAS status", kind: "status", service: "nas" },
      { id: "send", label: "Send the report", kind: "notify", text: "{{vars.report}}: the NAS is {{steps.nas.state}}" },
    ]),
  },
  { name: "empty", draft: draft("", "", [], []) },
];

export function templateNamed(name: string | null): WorkflowTemplate | undefined {
  return TEMPLATES.find((template) => template.name === name);
}
