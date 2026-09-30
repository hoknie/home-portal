export const VARIABLE_PREFIX = "STEP_";

const VARIABLE_NAME = /^STEP_[A-Z0-9_]{1,59}$/;

export function variableProblem(name: string): "variableName" | null {
  return VARIABLE_NAME.test(name) ? null : "variableName";
}
