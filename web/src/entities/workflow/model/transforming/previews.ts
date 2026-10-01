import type { z } from "zod";

import { generated } from "@/shared/api";

import type { Condition } from "../schema";
import { templateNames } from "../suggestions/check";
import type { Operation } from "./operations";

export const transformPreviewSchema = generated.transformPreview.schema;

export type PreviewAnswer = z.infer<typeof transformPreviewSchema>;

export type PreviewQuestion = { value: unknown; filters: string; operations: Operation[]; examples: string[]; names: Record<string, unknown> };

export type PreviewState = { state: "computing" } | { state: "unreachable" } | { state: "refused"; message: string } | { state: "ready"; answer: PreviewAnswer };

export type PreviewCache = (question: PreviewQuestion) => PreviewState;

export type Preview = { value: unknown; error: string | null; unknown?: string; waiting?: "computing" | "unreachable" };

export type ChainPreview = { input: Preview; steps: Preview[]; examples: Preview[] };

export type KnownValue = (name: string) => { value: unknown } | null;

const SENT_NAMESPACES = ["steps", "vars", "inputs"];
const OWN_NAMESPACES = ["item", "index"];

function conditionTexts(condition: Condition | undefined): string[] {
  if (condition === undefined) {
    return [];
  }
  return [condition.left ?? "", condition.right ?? "", ...(condition.all ?? []).flatMap(conditionTexts), ...(condition.any ?? []).flatMap(conditionTexts)];
}

function textsOf(operation: Operation): string[] {
  const args = (operation.args ?? []).filter((argument): argument is string => typeof argument === "string");
  return [...args, operation.to ?? "", operation.key ?? "", ...conditionTexts(operation.where), ...(operation.operations ?? []).flatMap(textsOf)];
}

export function namesOf(operation: Operation): string[] {
  return textsOf(operation)
    .flatMap(templateNames)
    .filter((found) => found.valid && !OWN_NAMESPACES.includes(found.name.split(".")[0]))
    .map((found) => found.name);
}

export function settled(preview: Preview | undefined): preview is Preview {
  return preview !== undefined && preview.error === null && preview.waiting === undefined;
}

function fromOutcome(outcome: PreviewAnswer["input"] | undefined, previous: Preview | undefined): Preview {
  if (outcome === undefined) {
    return previous && previous.error !== null ? previous : { value: null, error: null, waiting: "computing" };
  }
  return typeof outcome.error === "string" ? { value: null, error: outcome.error } : { value: outcome.value ?? null, error: null };
}

function every(count: number, preview: Preview): Preview[] {
  return Array.from({ length: count }, () => preview);
}

export function askPreview(cache: PreviewCache | undefined, question: Omit<PreviewQuestion, "names">, known: KnownValue): ChainPreview {
  const names: Record<string, unknown> = {};
  let unknownAt = question.operations.length;
  let unknownName = "";
  question.operations.forEach((operation, index) => {
    for (const name of namesOf(operation)) {
      const sample = SENT_NAMESPACES.includes(name.split(".")[0]) ? known(name) : null;
      if (sample !== null) {
        names[name] = sample.value;
      } else if (index < unknownAt) {
        unknownAt = index;
        unknownName = name;
      }
    }
  });
  const state: PreviewState = cache ? cache({ ...question, names }) : { state: "unreachable" };
  const count = question.operations.length;
  if (state.state === "computing" || state.state === "unreachable") {
    const waiting: Preview = { value: null, error: null, waiting: state.state };
    return { input: waiting, steps: every(count, waiting), examples: every(question.examples.length, waiting) };
  }
  if (state.state === "refused") {
    const refused: Preview = { value: null, error: state.message };
    return { input: refused, steps: every(count, refused), examples: every(question.examples.length, refused) };
  }
  const input = fromOutcome(state.answer.input, undefined);
  const steps: Preview[] = [];
  for (let index = 0; index < count; index += 1) {
    steps.push(index >= unknownAt ? { value: null, error: `${unknownName} is known after a run`, unknown: unknownName } : fromOutcome(state.answer.steps[index], steps.at(-1)));
  }
  const examples = question.examples.map((_, index) => fromOutcome(state.answer.examples[index], undefined));
  return { input, steps, examples };
}
