import { type Path, type Step, type Target, contains, everyStep, listsOf, pathText, stepsIn } from "@/entities/workflow";

export type Destination = { target: Target; owner: Step | null; list: string };

export function destinationsFor(steps: Step[], path: Path): Destination[] {
  const found: Destination[] = [];
  const parent = pathText(path.slice(0, -1));
  const own = path.at(-1)?.list;
  if (path.length > 1) {
    found.push({ target: { owner: [], list: "steps", index: steps.length }, owner: null, list: "steps" });
  }
  everyStep(steps, (step, here) => {
    if (contains(path, here)) {
      return;
    }
    for (const list of listsOf(step)) {
      if (pathText(here) === parent && list === own) {
        continue;
      }
      found.push({ target: { owner: here, list, index: stepsIn(steps, here, list).length }, owner: step, list });
    }
  });
  return found;
}
