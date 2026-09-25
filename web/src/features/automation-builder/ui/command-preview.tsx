"use client";

import { useTranslations } from "next-intl";

import type { CatalogueEvent } from "@/entities/automation";
import { SectionCard } from "@/shared/ui/section-card";

import { samplesOf } from "../model/events";
import { commandLineOf, variableOf } from "../model/preview";

export type CommandPreviewProps = {
  event: CatalogueEvent;
  script: string;
  args: string[];
  chosen: Record<string, string | undefined>;
};

export function CommandPreview({ event, script, args, chosen }: CommandPreviewProps) {
  const t = useTranslations("automationBuilder");
  const samples = samplesOf(event, chosen);
  return (
    <SectionCard title={t("preview")} description={t("previewDescription")}>
      <div className="grid gap-4">
        <pre data-testid="command-line" className="overflow-x-auto rounded-md border border-glass-edge bg-glass-tint p-3 font-mono text-xs whitespace-pre-wrap break-all">
          {commandLineOf(script === "" ? t("noScript") : script, args, samples)}
        </pre>
        <details>
          <summary className="cursor-pointer text-sm font-medium">{t("variables", { count: event.fields.length })}</summary>
          <dl className="mt-2 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 font-mono text-xs">
            {event.fields.map((field) => (
              <div key={field.name} className="contents">
                <dt className="text-muted-foreground">{variableOf(field.name)}</dt>
                <dd className="break-all">{samples[field.name]}</dd>
              </div>
            ))}
          </dl>
        </details>
      </div>
    </SectionCard>
  );
}
