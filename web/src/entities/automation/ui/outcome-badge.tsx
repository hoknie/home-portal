"use client";

import { useTranslations } from "next-intl";

import type { Outcome } from "../model/schema";
import { Badge } from "@/shared/ui/primitives";

const VARIANT: Record<Outcome, "default" | "secondary" | "destructive" | "outline"> = {
  succeeded: "default",
  failed: "destructive",
  "timed-out": "destructive",
  refused: "destructive",
  skipped: "secondary",
  unknown: "outline",
};

export function OutcomeBadge({ outcome }: { outcome: Outcome }) {
  const t = useTranslations("automations.outcomes");
  return (
    <Badge variant={VARIANT[outcome]} data-outcome={outcome}>
      {t(outcome.replace("-", "_") as "timed_out")}
    </Badge>
  );
}
