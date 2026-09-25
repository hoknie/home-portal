"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useState } from "react";

import { Label, Switch } from "@/shared/ui/primitives";

import type { PreviewScope } from "../model/catalog";
import { SELECT } from "./section-block";

export type PreviewPanelProps = { environments: string[]; render: (scope: PreviewScope) => ReactNode };

export function PreviewPanel({ environments, render }: PreviewPanelProps) {
  const t = useTranslations("layoutEditor");
  const [scope, setScope] = useState<PreviewScope>({ environment: environments[0] ?? "internet", signedIn: true });
  return (
    <details className="glass-panel rounded-xl p-4">
      <summary className="cursor-pointer text-sm font-medium select-none">{t("preview")}</summary>
      <div className="mt-4 grid gap-4">
        <div className="flex flex-wrap items-center gap-4">
          <label className="flex items-center gap-2 text-sm">
            {t("previewEnvironment")}
            <select className={SELECT} value={scope.environment} onChange={(event) => setScope({ ...scope, environment: event.target.value })}>
              {environments.map((name) => (
                <option key={name} value={name}>
                  {name}
                </option>
              ))}
            </select>
          </label>
          <div className="flex items-center gap-2">
            <Switch id="preview-signed-in" checked={scope.signedIn} onCheckedChange={(signedIn) => setScope({ ...scope, signedIn })} />
            <Label htmlFor="preview-signed-in">{t("previewSignedIn")}</Label>
          </div>
        </div>
        <div className="rounded-xl border border-dashed border-glass-edge p-4">{render(scope)}</div>
      </div>
    </details>
  );
}
