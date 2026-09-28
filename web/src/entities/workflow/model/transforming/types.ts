import type { Scope } from "../scope";
import { FILTERS, takes } from "./filters";
import type { FilterCall } from "./parse";
import type { ValueType } from "./values";

const RESULT_TYPES: Record<string, Record<string, ValueType>> = {
  http: { status: "number", body: "text", headers: "object" },
  script: { exit_code: "number", stdout: "text", stderr: "text" },
  probe: { latency_milliseconds: "number", state: "text", diagnosis: "text" },
  status: { state: "text", since: "text" },
  loop: { iterations: "number" },
  if: { branch: "text" },
  workflow: { vars: "object" },
};

export function resultType(kind: string, field: string): ValueType {
  return RESULT_TYPES[kind]?.[field] ?? "any";
}

export function certainType(name: string, scope: Scope): ValueType {
  const parts = name.split(".");
  const [namespace, first] = parts;
  if (namespace === "inputs" && parts.length === 2) {
    return scope.inputTypes[first] ?? "text";
  }
  if (namespace === "secrets" && parts.length === 2) {
    return "text";
  }
  if (namespace === "portal") {
    const field = parts.at(-1) ?? "";
    if (parts.length === 2 && (first === "services" || first === "environments")) {
      return "list";
    }
    if (field === "is_enabled" || (first === "services" && field === "public")) {
      return "boolean";
    }
    if ((first === "services" && field === "latency_milliseconds") || (first === "network" && field === "port")) {
      return "number";
    }
    return (first === "services" && parts.length === 4) || (first === "network" && parts.length === 3) ? "text" : "any";
  }
  if (namespace === "event") {
    return "text";
  }
  if ((namespace === "loop" && first === "index") || name === "index") {
    return "number";
  }
  if (namespace === "steps" && parts.length === 3) {
    const kind = scope.steps.find((step) => step.id === first)?.kind;
    return kind ? resultType(kind, parts[2]) : "any";
  }
  return "any";
}

export type FilterProblem =
  | { reason: "unknownFilter"; params: { filter: string } }
  | { reason: "filterArguments"; params: { filter: string; wanted: string; got: string } }
  | { reason: "filterArgument"; params: { filter: string; argument: string } }
  | { reason: "filterType"; params: { filter: string; takes: string; got: string } };

export function givesAfter(start: ValueType, filters: FilterCall[]): ValueType {
  return filters.reduce<ValueType>((current, call) => {
    const filter = FILTERS[call.name];
    if (!filter) {
      return "any";
    }
    if (call.name === "slice") {
      return current;
    }
    return current === "list" && filter.element && !takes(filter.accepts, "list") ? "list" : filter.gives;
  }, start);
}

export function chainProblem(start: ValueType, filters: FilterCall[]): FilterProblem | null {
  let current = start;
  for (const call of filters) {
    const filter = FILTERS[call.name];
    if (!filter) {
      return { reason: "unknownFilter", params: { filter: call.name } };
    }
    const fewest = filter.arguments.filter((argument) => argument.required).length;
    const most = filter.arguments.length;
    if (call.arguments.length < fewest || call.arguments.length > most) {
      return {
        reason: "filterArguments",
        params: { filter: call.name, wanted: fewest === most ? String(most) : `${fewest}–${most}`, got: String(call.arguments.length) },
      };
    }
    const variable = (index: number) => (call.names ?? []).some((named) => named.position === index) || (typeof call.arguments[index] === "string" && String(call.arguments[index]).includes("{{"));
    const wrong = filter.arguments.find(
      (argument, index) =>
        index < call.arguments.length &&
        !variable(index) &&
        ((argument.type === "text" && typeof call.arguments[index] !== "string") || (argument.type === "number" && typeof call.arguments[index] !== "number")),
    );
    if (wrong) {
      return { reason: "filterArgument", params: { filter: call.name, argument: wrong.name } };
    }
    const overList = current === "list" && filter.element;
    if (current !== "any" && current !== "null" && !overList && !takes(filter.accepts, current)) {
      return { reason: "filterType", params: { filter: call.name, takes: filter.accepts.join(" | "), got: current } };
    }
    current = call.name === "slice" ? current : overList && !takes(filter.accepts, "list") ? "list" : filter.gives;
  }
  return null;
}
