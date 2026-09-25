"use client";

import { LoaderCircle, RefreshCw } from "lucide-react";
import { useTranslations } from "next-intl";

import type { Service } from "@/entities/service";
import { Button } from "@/shared/ui/primitives";

import { useProbeNow } from "../model/use-probe-now";

export function ProbeNowButton({ service }: { service: Service }) {
  const t = useTranslations("probeNow");
  const probe = useProbeNow(service);
  return (
    <div className="flex flex-col items-start gap-1">
      <Button variant="outline" size="sm" onClick={probe.start} disabled={probe.busy || probe.disabled} aria-busy={probe.busy}>
        {probe.busy ? <LoaderCircle className="size-4 animate-spin" aria-hidden /> : <RefreshCw className="size-4" aria-hidden />}
        {probe.busy ? t("running") : t("start")}
      </Button>
      {probe.disabled ? <p className="text-xs text-muted-foreground">{t("disabled")}</p> : null}
      {probe.retryAfter !== null ? (
        <p role="status" className="text-xs text-muted-foreground">
          {t("throttled", { seconds: probe.retryAfter })}
        </p>
      ) : null}
      {probe.failed ? (
        <p role="alert" className="text-xs text-destructive">
          {t("failed")}
        </p>
      ) : null}
    </div>
  );
}
