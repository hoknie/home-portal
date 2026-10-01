"use client";

import { cn } from "cn";
import { useTranslations } from "next-intl";

import { OutcomeBadge, RunPager, useRunPages, useRuns } from "@/entities/automation";
import { Skeleton } from "@/shared/ui/primitives";
import { RelativeTime } from "@/shared/ui/relative-time";

import { useEditor } from "../../model/editor-context";

export const MANUAL_PREFIX = "workflow:";

export function RunsPanel({ onChoose }: { onChoose: (id: string) => void }) {
  const t = useTranslations("workflowEditor.runs");
  const editor = useEditor();
  const pages = useRunPages(editor.workflowId ?? "");
  const runs = useRuns({ workflow: editor.workflowId, before: pages.before }, true, editor.workflowId !== null);
  const starter = (automation: string, by: string | undefined) => {
    if (automation.startsWith(MANUAL_PREFIX)) {
      return by ? t("byPerson", { name: by }) : t("byHand");
    }
    return editor.usedBy.find((usage) => usage.id === automation)?.title ?? automation;
  };
  if (editor.workflowId === null) {
    return <p className="p-3 text-sm text-muted-foreground">{t("notSaved")}</p>;
  }
  if (!runs.data) {
    return <Skeleton className="m-3 h-24" aria-busy="true" />;
  }
  if (runs.data.runs.length === 0) {
    return <p className="p-3 text-sm text-muted-foreground">{t("none")}</p>;
  }
  return (
    <div className="grid">
      <ul className="grid gap-1 p-2" aria-label={t("title")}>
        {runs.data.runs.map((run) => (
          <li key={run.id}>
            <button
              type="button"
              onClick={() => onChoose(run.id)}
              className={cn("grid w-full grid-cols-[auto_1fr_auto] items-center gap-x-3 rounded-lg p-2 text-start text-sm hover:bg-accent", editor.run?.id === run.id && "bg-accent")}
            >
              <OutcomeBadge outcome={run.outcome.result} />
              <span className="grid min-w-0">
                <span className="truncate font-medium">{starter(run.automation, run.fields["run.by"])}</span>
                <span className="text-xs text-muted-foreground">
                  <RelativeTime moment={run.started_at} />
                </span>
              </span>
              <span className="text-xs text-muted-foreground tabular-nums">{t("duration", { milliseconds: run.outcome.duration_milliseconds })}</span>
            </button>
          </li>
        ))}
      </ul>
      <RunPager pages={pages} nextBefore={runs.data.next_before} />
    </div>
  );
}
