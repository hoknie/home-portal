import { routes } from "@/shared/config";

export type WorkflowAddress =
  | { kind: "list" }
  | { kind: "new" }
  | { kind: "view"; id: string }
  | { kind: "edit"; id: string }
  | { kind: "history"; id: string }
  | { kind: "run"; id: string; run: string }
  | { kind: "unknown"; id: string };

const EDIT = "edit";
const HISTORY = "history";
const NEW = "new";

function decoded(segment: string) {
  try {
    return decodeURIComponent(segment);
  } catch {
    return segment;
  }
}

export function workflowAddressOf(pathname: string): WorkflowAddress {
  const base = routes.adminWorkflows.split("/").filter(Boolean);
  const segments = pathname.split("/").filter(Boolean);
  if (base.some((segment, index) => segments[index] !== segment)) {
    return { kind: "list" };
  }
  const [first, second, third, ...rest] = segments.slice(base.length).map(decoded);
  if (first === undefined) {
    return { kind: "list" };
  }
  if (first === NEW && second === undefined) {
    return { kind: "new" };
  }
  const id = first;
  if (rest.length > 0) {
    return { kind: "unknown", id };
  }
  if (second === undefined) {
    return { kind: "view", id };
  }
  if (second === EDIT && third === undefined) {
    return { kind: "edit", id };
  }
  if (second === HISTORY) {
    return third === undefined ? { kind: "history", id } : { kind: "run", id, run: third };
  }
  return { kind: "unknown", id };
}
