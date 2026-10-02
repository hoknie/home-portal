"use client";

import { useTranslations } from "next-intl";

import { Badge } from "@/shared/ui/kit";

import type { PermissionState } from "../model/schema";

const VARIANTS = {
  granted: "default",
  denied: "destructive",
  pending: "secondary",
  "not-applicable": "outline",
  unknown: "outline",
} as const satisfies Record<PermissionState, "default" | "destructive" | "secondary" | "outline">;

export function PermissionStateBadge({ state }: { state: PermissionState }) {
  const t = useTranslations("permissions.states");
  return (
    <Badge variant={VARIANTS[state]} data-state={state}>
      {t(state)}
    </Badge>
  );
}
