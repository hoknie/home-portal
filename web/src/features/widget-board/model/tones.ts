import type { Tone } from "@/entities/widget";

export const TONE_TEXT: Record<Tone, string> = {
  neutral: "text-foreground",
  ok: "text-tone-ok",
  info: "text-tone-info",
  warning: "text-tone-warning",
  danger: "text-tone-danger",
};

export const TONE_FILL: Record<Tone, string> = {
  neutral: "bg-primary",
  ok: "bg-tone-ok",
  info: "bg-tone-info",
  warning: "bg-tone-warning",
  danger: "bg-tone-danger",
};

export const TONE_BADGE: Record<Tone, string> = {
  neutral: "border-border bg-muted text-muted-foreground",
  ok: "border-tone-ok/30 bg-tone-ok/10 text-tone-ok",
  info: "border-tone-info/30 bg-tone-info/10 text-tone-info",
  warning: "border-tone-warning/30 bg-tone-warning/10 text-tone-warning",
  danger: "border-tone-danger/30 bg-tone-danger/10 text-tone-danger",
};

export function percentOf(value: number, maximum: number) {
  if (!(maximum > 0)) {
    return 0;
  }
  return Math.min(100, Math.max(0, (value / maximum) * 100));
}
