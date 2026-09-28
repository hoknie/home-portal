import type { InputDeclaration, InputType } from "./schema";

export function plainInput(name: string): InputDeclaration {
  return { name, type: "text", default: null, description: null };
}

export function namedInputs(inputs: InputDeclaration[]): InputDeclaration[] {
  return inputs.filter((input) => input.name.trim() !== "");
}

export function inputNames(inputs: InputDeclaration[]): string[] {
  return namedInputs(inputs).map((input) => input.name.trim());
}

export function emptyValue(type: InputType): unknown {
  switch (type) {
    case "number":
      return null;
    case "boolean":
      return false;
    case "list":
      return [];
    case "object":
      return {};
    default:
      return "";
  }
}

export function fitsType(type: InputType, value: unknown): boolean {
  switch (type) {
    case "text":
      return typeof value === "string";
    case "number":
      return typeof value === "number" && Number.isFinite(value);
    case "boolean":
      return typeof value === "boolean";
    case "list":
      return Array.isArray(value);
    default:
      return value !== null && typeof value === "object" && !Array.isArray(value);
  }
}

export function initialValues(inputs: InputDeclaration[]): Record<string, unknown> {
  return Object.fromEntries(namedInputs(inputs).map((input) => [input.name, input.default ?? emptyValue(input.type)]));
}

export function runValues(inputs: InputDeclaration[], values: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(
    namedInputs(inputs)
      .map((input) => [input.name, values[input.name]] as const)
      .filter(([, value]) => value !== null && value !== undefined && value !== ""),
  );
}
