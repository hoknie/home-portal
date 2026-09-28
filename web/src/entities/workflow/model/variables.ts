export const PASSED_THROUGH = ["PATH", "HOME", "LANG", "TZ"];

export const VARIABLE_PREFIX = "PORTAL_";

const VARIABLE_NAME = /^[A-Z][A-Z0-9_]{0,63}$/;

export function variableProblem(name: string): "variableName" | "variableReserved" | null {
  if (!VARIABLE_NAME.test(name)) {
    return "variableName";
  }
  return name.startsWith(VARIABLE_PREFIX) || PASSED_THROUGH.includes(name) ? "variableReserved" : null;
}
