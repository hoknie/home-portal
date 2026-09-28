import type { Trace } from "@/entities/automation/@x/workflow";

export function lastOutput(trace: Trace | null, step: string): string | null {
  const last = (trace?.entries ?? []).filter((entry) => entry.step === step && (entry.shape || entry.output)).at(-1);
  return last ? (last.shape ?? last.output) : null;
}
