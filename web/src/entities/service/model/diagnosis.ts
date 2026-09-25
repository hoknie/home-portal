import type { Diagnosis } from "@/shared/api";

export const DIAGNOSIS_MESSAGES = {
  "local-network-denied": { title: "diagnosis.local-network-denied.title", action: "diagnosis.local-network-denied.action" },
  refused: { title: "diagnosis.refused.title", action: "diagnosis.refused.action" },
  timeout: { title: "diagnosis.timeout.title", action: "diagnosis.timeout.action" },
  "host-unreachable": { title: "diagnosis.host-unreachable.title", action: "diagnosis.host-unreachable.action" },
  "name-not-resolved": { title: "diagnosis.name-not-resolved.title", action: "diagnosis.name-not-resolved.action" },
  tls: { title: "diagnosis.tls.title", action: "diagnosis.tls.action" },
  "not-http": { title: "diagnosis.not-http.title", action: "diagnosis.not-http.action" },
  "http-status": { title: "diagnosis.http-status.title", action: "diagnosis.http-status.action" },
  "icmp-not-permitted": { title: "diagnosis.icmp-not-permitted.title", action: "diagnosis.icmp-not-permitted.action" },
  other: { title: "diagnosis.other.title", action: "diagnosis.other.action" },
} as const satisfies Record<Diagnosis, { title: string; action: string }>;

export function diagnosisMessage(code: Diagnosis | null) {
  return code ? DIAGNOSIS_MESSAGES[code] : null;
}
