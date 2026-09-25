export const KNOWN_STATES = ["unknown", "up", "degraded", "down", "unreadable"] as const;

export type KnownState = (typeof KNOWN_STATES)[number];

export function knownState(value: string): KnownState {
  return (KNOWN_STATES as readonly string[]).includes(value) ? (value as KnownState) : "unknown";
}

export const STATE_DOT: Record<KnownState, string> = {
  unknown: "bg-status-unknown",
  up: "bg-status-up",
  degraded: "bg-status-degraded",
  down: "bg-status-down",
  unreadable: "bg-status-unreadable",
};

export const STATE_TONE: Record<KnownState, string> = {
  unknown: "text-status-unknown bg-status-unknown/10 border-status-unknown/25",
  up: "text-status-up bg-status-up/10 border-status-up/25",
  degraded: "text-status-degraded bg-status-degraded/10 border-status-degraded/25",
  down: "text-status-down bg-status-down/10 border-status-down/25",
  unreadable: "text-status-unreadable bg-status-unreadable/10 border-status-unreadable/25",
};
