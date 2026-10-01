"use client";

import { RefreshCw } from "lucide-react";
import { useFormatter, useTranslations } from "next-intl";
import { useEffect, useState } from "react";

import { RunSource, useItemReferences } from "@/features/runs/item-references";
import { StopRunButton } from "@/features/runs/stop-run";
import { type Automation, RunPager, RunTable, useRunPages, useRuns } from "@/entities/automation";
import { useWebhooks } from "@/entities/webhook";
import { useWorkflows } from "@/entities/workflow";
import { Button, Input, Label, Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectTrigger, SelectValue, Skeleton, Switch } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

export const EVERYTHING = "all";
export const TEXT_DEBOUNCE_MILLISECONDS = 300;
const AUTOMATION = "automation:";
const WEBHOOK = "webhook:";
const WORKFLOW = "workflow:";

function useSettled(value: string) {
  const [settled, setSettled] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => setSettled(value), TEXT_DEBOUNCE_MILLISECONDS);
    return () => window.clearTimeout(timer);
  }, [value]);
  return settled;
}

export function RunJournal({ automations, onOpen }: { automations: Automation[]; onOpen?: (id: string) => void }) {
  const t = useTranslations();
  const [source, setSource] = useState(EVERYTHING);
  const [text, setText] = useState("");
  const [live, setLive] = useState(true);
  const settled = useSettled(text.trim());
  const references = useItemReferences();
  const webhooks = useWebhooks();
  const hooks = webhooks.data?.data.webhooks ?? [];
  const flows = useWorkflows().data?.data.workflows ?? [];
  const pages = useRunPages(`${source}\n${settled}`);
  const runs = useRuns({
    automation: source.startsWith(AUTOMATION) ? source.slice(AUTOMATION.length) : null,
    webhook: source.startsWith(WEBHOOK) ? source.slice(WEBHOOK.length) : null,
    workflow: source.startsWith(WORKFLOW) ? source.slice(WORKFLOW.length) : null,
    text: settled || null,
    before: pages.before,
  }, live);
  const format = useFormatter();
  const [lastUpdate, setLastUpdate] = useState(0);
  if (runs.dataUpdatedAt > 0 && runs.dataUpdatedAt !== lastUpdate) {
    setLastUpdate(runs.dataUpdatedAt);
  }
  const updated = lastUpdate > 0 ? new Date(lastUpdate) : null;
  const [refreshing, setRefreshing] = useState(false);
  const refresh = async () => {
    setRefreshing(true);
    await runs.refetch();
    setRefreshing(false);
  };
  const titleOf = (id: string) =>
    automations.find((automation) => automation.id === id)?.title ??
    hooks.find((webhook) => webhook.id === id)?.title ??
    (id.startsWith(WORKFLOW) ? flows.find((workflow) => `${WORKFLOW}${workflow.id}` === id)?.title : undefined) ??
    id;
  const filters = (
    <div className="flex flex-wrap gap-2">
      <Select value={source} onValueChange={setSource}>
        <SelectTrigger aria-label={t("automations.filterLabel")} className="w-56">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={EVERYTHING}>{t("automations.allRuns")}</SelectItem>
          {automations.length > 0 ? (
            <SelectGroup>
              <SelectLabel>{t("automations.title")}</SelectLabel>
              {automations.map((automation) => (
                <SelectItem key={automation.id} value={`${AUTOMATION}${automation.id}`}>
                  {automation.title}
                </SelectItem>
              ))}
            </SelectGroup>
          ) : null}
          {hooks.length > 0 ? (
            <SelectGroup>
              <SelectLabel>{t("webhooks.title")}</SelectLabel>
              {hooks.map((webhook) => (
                <SelectItem key={webhook.id} value={`${WEBHOOK}${webhook.id}`}>
                  {webhook.title}
                </SelectItem>
              ))}
            </SelectGroup>
          ) : null}
          {flows.length > 0 ? (
            <SelectGroup>
              <SelectLabel>{t("workflows.title")}</SelectLabel>
              {flows.map((workflow) => (
                <SelectItem key={workflow.id} value={`${WORKFLOW}${workflow.id}`}>
                  {workflow.title}
                </SelectItem>
              ))}
            </SelectGroup>
          ) : null}
        </SelectContent>
      </Select>
      <Input
        aria-label={t("automations.textFilter")}
        placeholder={t("automations.textFilterPlaceholder")}
        className="w-56"
        value={text}
        onChange={(change) => setText(change.target.value)}
      />
      <div className="flex items-center gap-2">
        <Switch id="journal-live" checked={live} onCheckedChange={setLive} />
        <Label htmlFor="journal-live" className="text-sm font-normal">
          {t("automations.live")}
        </Label>
      </div>
      <Button type="button" variant="ghost" size="icon" aria-label={t("automations.refresh")} disabled={refreshing} onClick={() => void refresh()}>
        <RefreshCw className={refreshing ? "animate-spin" : undefined} aria-hidden />
      </Button>
      <span className="min-w-36 self-center text-xs text-muted-foreground tabular-nums" aria-live="off">
        {updated ? t("automations.updatedAt", { time: format.dateTime(updated, { timeStyle: "medium" }) }) : null}
      </span>
    </div>
  );
  return (
    <SectionCard title={t("automations.journal")} description={t("automations.journalDescription")} actions={filters} flush>
      {runs.data ? (
        <>
          <RunTable
            runs={runs.data.runs}
            sourceOf={(run) => <RunSource run={run} references={references} />}
            onOpen={onOpen}
            actionsOf={(run) => <StopRunButton run={run} title={titleOf(run.automation)} />}
          />
          <RunPager pages={pages} nextBefore={runs.data.next_before} />
        </>
      ) : <Skeleton className="m-4 h-32" aria-busy="true" />}
    </SectionCard>
  );
}
