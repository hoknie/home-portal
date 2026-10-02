"use client";

import { Globe, TriangleAlert } from "lucide-react";
import { useTranslations } from "next-intl";

import { EnvironmentPicker, useEnvironment } from "@/entities/environment";
import { cn } from "@/shared/lib/cn";
import { Label, Switch } from "@/shared/ui/primitives";

export type AccessValue = { environments: string[] | null; public: boolean };

const INTERNET = "internet";

export function AccessTab({ value, environments, custom, onChange }: { value: AccessValue; environments: string[]; custom: boolean; onChange: (value: AccessValue) => void }) {
  const t = useTranslations("layoutEditor");
  const current = useEnvironment().data?.environment ?? null;
  const unseenOutside = value.public && value.environments !== null && !value.environments.includes(INTERNET);
  return (
    <div className="grid gap-6">
      <section className="grid gap-3" aria-labelledby="access-environments">
        <div className="grid gap-1">
          <h3 id="access-environments" className="text-sm font-medium">
            {t("environments")}
          </h3>
          <p className="text-xs text-muted-foreground">{t("environmentsHint")}</p>
        </div>
        <EnvironmentPicker label={t("environments")} environments={environments} value={value.environments} current={current} onChange={(next) => onChange({ ...value, environments: next })} />
      </section>
      <section className={cn("grid gap-3 rounded-xl border p-4 transition-colors", value.public ? "border-primary/40 bg-primary/5" : "border-glass-edge bg-glass-tint")}>
        <div className="flex items-start gap-3">
          <span className={cn("grid size-9 shrink-0 place-items-center rounded-lg", value.public ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground")}>
            <Globe className="size-4" aria-hidden />
          </span>
          <div className="grid min-w-0 flex-1 gap-1">
            <Label htmlFor="widget-public">{t("public")}</Label>
            <p className="text-xs text-muted-foreground">{t("publicHint")}</p>
          </div>
          <Switch id="widget-public" checked={value.public} onCheckedChange={(checked) => onChange({ ...value, public: checked })} />
        </div>
        {unseenOutside ? (
          <p role="note" className="flex items-start gap-2 rounded-lg bg-status-degraded/10 px-3 py-2 text-xs">
            <TriangleAlert className="mt-0.5 size-3.5 shrink-0 text-status-degraded" aria-hidden />
            {t("publicNotOutside")}
          </p>
        ) : null}
        {custom && value.public ? <p className="text-xs text-muted-foreground">{t("dialog.publicCustom")}</p> : null}
      </section>
    </div>
  );
}
