"use client";

import { useTranslations } from "next-intl";

import { quotedCommand } from "@/shared/lib/shell-quote";
import { CopyLine } from "@/shared/ui/copy-line";
import { KvList, KvRow } from "@/shared/ui/kv-list";
import { Button, Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/shared/ui/primitives";

import type { TraceEntry } from "../../model/schema";
import { exitCodeOf, scriptLogOf } from "../../model/script-log";
import { OutcomeBadge } from "../outcome-badge";
import { RunOutput } from "../run-output";

export function ScriptLogDialog({ entry }: { entry: TraceEntry }) {
  const t = useTranslations("workflows.trace.scriptLog");
  const trace = useTranslations("workflows.trace");
  const log = scriptLogOf(entry);
  if (log === null) {
    return null;
  }
  const exitCode = exitCodeOf(entry);
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button type="button" variant="outline" size="sm" className="h-6 justify-self-start px-2 text-xs">
          {t("open")}
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>{t("title", { label: entry.label })}</DialogTitle>
          <DialogDescription className="break-all">{entry.detail}</DialogDescription>
        </DialogHeader>
        {log.kind === "streams" && log.command !== null && log.command.length > 0 ? (
          <CopyLine label={t("command")} text={quotedCommand(log.command[0], log.command.slice(1))} />
        ) : null}
        <KvList>
          <KvRow label={t("outcome")}>
            <OutcomeBadge outcome={entry.outcome} />
          </KvRow>
          {exitCode === null ? null : <KvRow label={t("exitCode")}>{exitCode}</KvRow>}
          <KvRow label={t("duration")}>{trace("duration", { value: entry.duration_milliseconds })}</KvRow>
        </KvList>
        {log.kind === "streams" ? (
          <>
            <RunOutput label={t("stdout")} output={log.stdout} copy budget={log.budgetReached} />
            <RunOutput label={t("stderr")} output={log.stderr} copy budget={log.budgetReached} />
          </>
        ) : (
          <RunOutput label={t("output")} output={log.output} copy />
        )}
      </DialogContent>
    </Dialog>
  );
}
