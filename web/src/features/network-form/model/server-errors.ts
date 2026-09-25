import type { FieldError } from "@/shared/api";

export function networkFieldOf(field: string) {
  return field.startsWith("trusted_proxies") ? "trusted_proxies" : field;
}

export function byField(errors: FieldError[]) {
  return errors.map((error) => ({ path: networkFieldOf(error.field), message: error.message }));
}
