import type { InputDeclaration, InputType, Step, WorkflowCatalogue } from "./schema";
import { type Path, ROOT, childList, listsOf } from "./tree";

export type Scope = {
  inputs: string[];
  inputTypes: Record<string, InputType>;
  vars: string[];
  setBy: Record<string, string>;
  steps: { id: string; kind: string }[];
  inLoop: boolean;
  inTransform: boolean;
};

type Walk = { vars: Set<string>; setBy: Record<string, string>; steps: { id: string; kind: string }[]; loops: number };

function samePath(left: Path, right: Path) {
  return left.length === right.length && left.every((place, index) => place.list === right[index].list && place.index === right[index].index);
}

function walk(steps: Step[], owner: Path, list: string, target: Path, field: string, state: Walk): Scope | null {
  for (const [index, step] of steps.entries()) {
    const here = [...owner, { list, index }];
    const isLoop = step.kind === "loop";
    if (samePath(here, target)) {
      const ownLoop = isLoop && field.startsWith("while");
      return {
        inputs: [],
        inputTypes: {},
        vars: [...state.vars],
        setBy: { ...state.setBy },
        steps: [...state.steps],
        inLoop: state.loops + (ownLoop ? 1 : 0) > 0,
        inTransform: step.kind === "transform" && field.startsWith("operations"),
      };
    }
    state.steps.push({ id: step.id, kind: step.kind });
    state.loops += isLoop ? 1 : 0;
    for (const child of listsOf(step)) {
      const found = walk(childList(step, child), here, child, target, field, state);
      if (found !== null) {
        return found;
      }
    }
    state.loops -= isLoop ? 1 : 0;
    if (step.kind === "set" && step.variable) {
      state.vars.add(step.variable);
      state.setBy[step.variable] = step.id;
    }
  }
  return null;
}

export const END_OF_WORKFLOW: Path = [{ list: ROOT, index: Number.MAX_SAFE_INTEGER }];

export function scopeAt(steps: Step[], path: Path, inputs: (string | InputDeclaration)[], field = ""): Scope {
  const state: Walk = { vars: new Set(), setBy: {}, steps: [], loops: 0 };
  const walked = walk(steps, [], ROOT, path, field, state);
  const found =
    walked ?? (samePath(path, END_OF_WORKFLOW) ? { inputs: [], inputTypes: {}, vars: [...state.vars], setBy: { ...state.setBy }, steps: [...state.steps], inLoop: false, inTransform: false } : null);
  const declared = inputs.map((input) => (typeof input === "string" ? { name: input, type: "text" as const } : input)).filter((input) => input.name.trim() !== "");
  return {
    ...(found ?? { vars: [], setBy: {}, steps: [], inLoop: false, inTransform: false }),
    inputs: declared.map((input) => input.name.trim()),
    inputTypes: Object.fromEntries(declared.map((input) => [input.name.trim(), input.type])),
  };
}

export function chipsOf(scope: Scope, catalogue: WorkflowCatalogue | undefined, eventFields: string[] = []): string[] {
  const results = (kind: string) => catalogue?.kinds.find((entry) => entry.name === kind)?.results ?? [];
  return [
    ...scope.inputs.map((name) => `inputs.${name}`),
    ...scope.vars.map((name) => `vars.${name}`),
    ...scope.steps.flatMap((step) => results(step.kind).map((result) => `steps.${step.id}.${result}`)),
    ...(scope.inLoop ? ["loop.item", "loop.index"] : []),
    ...(scope.inTransform ? ["item", "index"] : []),
    ...eventFields.map((name) => `event.${name}`),
  ];
}
