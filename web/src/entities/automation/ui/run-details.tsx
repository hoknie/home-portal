"use client";

import { useFormatter, useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { useRun } from "../model/queries";
import { type Run, messageKeyOf } from "../model/schema";
import { KvList, KvRow } from "@/shared/ui/kv-list";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle, Skeleton } from "@/shared/ui/primitives";

import { OutcomeBadge } from "./outcome-badge";
import { RunOutput } from "./run-output";

export const STOPPED_BY = "stopped by ";

export type RunDetailsProps = { runId: string | null; onClose: () => void; actions?: (run: Run) => ReactNode };

function Reason({ reason }: { reason: string }) {
  const t = useTranslations("automations");
  if (reason.startsWith(STOPPED_BY)) {
    return t("reasons.stoppedBy", { name: reason.slice(STOPPED_BY.length) });
  }
  const key = `reasons.${messageKeyOf(reason)}` as Parameters<typeof t.has>[0];
  return t.has(key) ? t(key as Parameters<typeof t>[0]) : reason;
}

function Body({ run, actions }: { run: Run; actions?: (run: Run) => ReactNode }) {
  const t = useTranslations();
  const format = useFormatter();
  return (
    <>
      <SheetHeader>
        <SheetTitle>{t("automations.runTitle", { id: run.id, automation: run.automation })}</SheetTitle>
        <SheetDescription>{format.dateTime(new Date(run.started_at), { dateStyle: "medium", timeStyle: "medium" })}</SheetDescription>
      </SheetHeader>
      <div className="grid gap-5 px-4 pb-6">
        {actions ? <div className="flex gap-2">{actions(run)}</div> : null}
        <KvList>
          <KvRow label={t("automations.runOutcome")}>
            <OutcomeBadge outcome={run.outcome.result} />
          </KvRow>
          <KvRow label={t("automations.runEvent")}>{t(`automationEvents.${messageKeyOf(run.event)}.title` as Parameters<typeof t>[0])}</KvRow>
          {run.outcome.exit_code !== null ? <KvRow label={t("automations.runExit")}>{run.outcome.exit_code}</KvRow> : null}
          {run.outcome.reason ? (
            <KvRow label={t("automations.runReason")}>
              <Reason reason={run.outcome.reason} />
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
        <RunOutput label={t("automations.runOutput")} output={run.outcome.stdout} />
        <RunOutput label={t("automations.runErrors")} output={run.outcome.stderr} />
      </div>
    </>
  );
}

export function RunDetails({ runId, onClose, actions }: RunDetailsProps) {
  const run = useRun(runId);
  return (
    <Sheet open={runId !== null} onOpenChange={(open) => (open ? undefined : onClose())}>
      <SheetContent className="w-full overflow-y-auto sm:max-w-xl">
        {runId === null ? null : run.data ? <Body run={run.data} actions={actions} /> : <Skeleton className="m-4 h-64" aria-busy="true" />}
      </SheetContent>
    </Sheet>
  );
}
