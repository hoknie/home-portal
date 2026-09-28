import { type Step, type WorkflowRequest, everyStep, pathText, templateNamed, updateAt } from "@/entities/workflow";

export type Translate = { has: (key: string) => boolean; text: (key: string) => string };

export function localizedTemplate(name: string | null, translate: Translate): WorkflowRequest | null {
  const template = templateNamed(name);
  if (!template) {
    return null;
  }
  const base = `templates.${template.name}`;
  let steps: Step[] = template.draft.steps;
  const paths: { id: string; path: Parameters<typeof updateAt>[1] }[] = [];
  everyStep(steps, (step, path) => paths.push({ id: step.id, path }));
  for (const { id, path } of paths) {
    const key = `${base}.labels.${id}`;
    if (translate.has(key)) {
      steps = updateAt(steps, path, (step) => ({ ...step, label: translate.text(key) }));
    }
  }
  const title = template.name === "empty" ? "" : translate.text(`${base}.title`);
  return { ...template.draft, title, steps };
}

export function nodeLabel(steps: Step[], at: string): string | null {
  let found: string | null = null;
  everyStep(steps, (step, path) => {
    if (pathText(path) === at) {
      found = step.label ?? step.id;
    }
  });
  return found;
}
