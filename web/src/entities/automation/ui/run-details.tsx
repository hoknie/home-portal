"use client";

import { useFormatter, useTranslations } from "next-intl";

import { type Run, messageKeyOf } from "../model/schema";
import { KvList, KvRow } from "@/shared/ui/kv-list";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/shared/ui/primitives";

import { OutcomeBadge } from "./outcome-badge";

export type RunDetailsProps = { run: Run | null; onClose: () => void };

function Output({ label, output }: { label: string; output: Run["outcome"]["stdout"] }) {
  const t = useTranslations("automations");
  return (
    <div className="grid gap-1">
      <p className="text-sm font-medium">{label}</p>
      {output.truncated ? <p className="text-xs text-muted-foreground">{t("truncated", { bytes: output.bytes })}</p> : null}
      <pre className="max-h-64 overflow-auto rounded-md border border-glass-edge bg-glass-tint p-3 font-mono text-xs whitespace-pre-wrap break-all">
        {output.tail === "" ? t("noOutput") : output.tail}
      </pre>
    </div>
  );
}

export function RunDetails({ run, onClose }: RunDetailsProps) {
  const t = useTranslations();
  const format = useFormatter();
  return (
    <Sheet open={run !== null} onOpenChange={(open) => (open ? undefined : onClose())}>
      <SheetContent className="w-full overflow-y-auto sm:max-w-xl">
        {run ? (
          <>
            <SheetHeader>
              <SheetTitle>{t("automations.runTitle", { id: run.id, automation: run.automation })}</SheetTitle>
              <SheetDescription>{format.dateTime(new Date(run.started_at), { dateStyle: "medium", timeStyle: "medium" })}</SheetDescription>
            </SheetHeader>
            <div className="grid gap-5 px-4 pb-6">
              <KvList>
                <KvRow label={t("automations.runOutcome")}>
                  <OutcomeBadge outcome={run.outcome.result} />
                </KvRow>
                <KvRow label={t("automations.runEvent")}>{t(`automationEvents.${messageKeyOf(run.event)}.title` as Parameters<typeof t>[0])}</KvRow>
                {run.outcome.exit_code !== null ? <KvRow label={t("automations.runExit")}>{run.outcome.exit_code}</KvRow> : null}
                {run.outcome.reason ? (
                  <KvRow label={t("automations.runReason")}>
                    {t.has(`automations.reasons.${messageKeyOf(run.outcome.reason)}` as Parameters<typeof t.has>[0])
                      ? t(`automations.reasons.${messageKeyOf(run.outcome.reason)}` as Parameters<typeof t>[0])
                      : run.outcome.reason}
                  </KvRow>
                ) : null}
                <KvRow label={t("automations.runDuration")}>{t("common.milliseconds", { value: run.outcome.duration_milliseconds })}</KvRow>
                {run.outcome.count > 1 ? <KvRow label={t("automations.runRepeated")}>{t("automations.repeated", { count: run.outcome.count })}</KvRow> : null}
              </KvList>
              <div className="grid gap-1">
                <p className="text-sm font-medium">{t("automations.runArguments")}</p>
                <ol className="grid gap-0.5 font-mono text-xs">
                  {run.arguments.map((argument, index) => (
                    <li key={`${index}-${argument}`} className="break-all">
                      {argument}
                    </li>
                  ))}
                </ol>
              </div>
              <div className="grid gap-1">
                <p className="text-sm font-medium">{t("automations.runFields")}</p>
                <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-0.5 font-mono text-xs">
                  {Object.entries(run.fields).map(([name, value]) => (
                    <div key={name} className="contents">
                      <dt className="text-muted-foreground">{name}</dt>
                      <dd className="break-all">{value}</dd>
                    </div>
                  ))}
                </dl>
              </div>
              <Output label={t("automations.runOutput")} output={run.outcome.stdout} />
              <Output label={t("automations.runErrors")} output={run.outcome.stderr} />
            </div>
          </>
        ) : null}
      </SheetContent>
    </Sheet>
  );
}
