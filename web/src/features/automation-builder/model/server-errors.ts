import type { FieldError } from "@/shared/api";

const WHEN = /^when\./;
const ARGUMENT = /^run\.args\[(\d+)\]$/;
const RUN = /^run\./;

export function formPathOf(field: string) {
  const argument = ARGUMENT.exec(field);
  if (argument) {
    return `args.${argument[1]}.value`;
  }
  return field.replace(WHEN, "").replace(RUN, "");
}

export function byField(errors: FieldError[]) {
  return errors.map((error) => ({ path: formPathOf(error.field), message: error.message }));
}
