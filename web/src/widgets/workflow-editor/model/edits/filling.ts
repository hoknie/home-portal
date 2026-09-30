import { type Step, childList, idFor, idsOf, listsOf, withChildList } from "@/entities/workflow";

export const NOTHING_KIND = "nothing";

function filled(step: Step, taken: Set<string>): Step {
  let copy = step;
  for (const list of listsOf(step)) {
    const children = childList(copy, list);
    if (children.length === 0) {
      const id = idFor(NOTHING_KIND, taken);
      taken.add(id);
      copy = withChildList(copy, list, [{ id, kind: NOTHING_KIND }]);
    } else {
      copy = withChildList(copy, list, children.map((child) => filled(child, taken)));
    }
  }
  return copy;
}

export function filledSteps(steps: Step[]): Step[] {
  const taken = idsOf(steps);
  return steps.map((step) => filled(step, taken));
}
