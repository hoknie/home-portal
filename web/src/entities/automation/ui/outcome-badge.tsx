"use client";

import { LoaderCircle } from "lucide-react";
import { useTranslations } from "next-intl";

import type { Outcome } from "../model/schema";
import { Badge } from "@/shared/ui/primitives";

const VARIANT: Record<Outcome, "default" | "secondary" | "destructive" | "outline"> = {
  queued: "outline",
  running: "outline",
  succeeded: "default",
  failed: "destructive",
  "timed-out": "destructive",
  stopped: "secondary",
  refused: "destructive",
  skipped: "secondary",
  unknown: "outline",
};

export function OutcomeBadge({ outcome }: { outcome: Outcome }) {
  const t = useTranslations("automations.outcomes");
  return (
    <Badge variant={VARIANT[outcome]} data-outcome={outcome}>
      {outcome === "running" ? <LoaderCircle className="animate-spin" aria-hidden /> : null}
      {t(outcome.replace("-", "_") as "timed_out")}
    </Badge>
  );
}
