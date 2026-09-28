export const ACTIONS = ["script", "workflow"] as const;

export type Action = (typeof ACTIONS)[number];

export type RunFields = {
  action: Action;
  script: string;
  args: { value: string }[];
  timeout_seconds: number;
  workflow: string;
  inputs: Record<string, string>;
};
