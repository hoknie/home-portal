import type { Automation } from "@/entities/automation";
import type { Webhook } from "@/entities/webhook";
import { type Workflow, everyStep } from "@/entities/workflow";

export type ScriptUsers = {
  automations: Automation[];
  webhooks: Webhook[];
  workflows: Workflow[];
};

export function usersOf(path: string, users: ScriptUsers): string[] {
  const found = [
    ...users.automations.filter((automation) => automation.run?.script === path && !automation.workflow).map((automation) => automation.title),
    ...users.webhooks.filter((webhook) => webhook.action === "script" && webhook.run?.script === path).map((webhook) => webhook.title),
  ];
  for (const workflow of users.workflows) {
    let named = false;
    everyStep(workflow.steps, (step) => {
      named = named || (step.kind === "script" && step.script === path);
    });
    if (named) {
      found.push(workflow.title);
    }
  }
  return found;
}

export const NEW_SCRIPT = "#!/bin/sh\n# @description What this script does\n# @arg service <text> Service id\nset -eu\n\n";

export function joined(folder: string, name: string) {
  return folder === "" ? name : `${folder}/${name}`;
}
