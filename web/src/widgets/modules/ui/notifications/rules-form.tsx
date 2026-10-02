"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { Allowed, useCan } from "@/entities/session";
import { type Rules, useChangeRules } from "@/entities/notification";
import { ConflictError, ValidationError } from "@/shared/api";
import { useEditorRevision } from "@/shared/lib/editor-revision";
import { ConflictNotice } from "@/shared/ui/conflict-notice";
import { Button, Checkbox, ErrorNotice, Label, Switch } from "@/shared/ui/kit";
import { SectionCard } from "@/shared/ui/section-card";

export const ANNOUNCED_STATES = ["down", "unreadable", "degraded", "up"] as const;

export function RulesForm({ rules, revision }: { rules: Rules; revision: string | null }) {
  const t = useTranslations();
  const can = useCan();
  const editable = can("notifications", "update");
  const change = useChangeRules();
  const [draft, setDraft] = useState(rules);
  const [problem, setProblem] = useState<string | null>(null);
  const [conflict, setConflict] = useState(false);
  const held = useEditorRevision(revision);
  const toggle = (state: string, on: boolean) =>
    setDraft((current) => ({ ...current, states: on ? [...current.states, state] : current.states.filter((name) => name !== state) }));
  const save = async (at: string | null = held.revision) => {
    setProblem(null);
    setConflict(false);
    try {
      const saved = await change.mutateAsync({ rules: draft, revision: at });
      held.adopt(saved.revision);
      setDraft(saved.data.rules);
      toast.success(t("notifications.saved"));
    } catch (error) {
      if (error instanceof ValidationError) {
        setProblem(error.fields.map((field) => field.message).join("; "));
      } else if (error instanceof ConflictError) {
        setConflict(true);
      } else {
        toast.error(t("errors.generic"));
      }
    }
  };
  return (
    <SectionCard title={t("notifications.rules.title")} description={t("notifications.rules.description")}>
      <div className="grid gap-5">
        {conflict ? (
          <ConflictNotice
            pending={held.latest === held.revision || change.isPending}
            onReload={() => {
              held.catchUp();
              setDraft(rules);
              setConflict(false);
            }}
            onOverwrite={() => void save(held.catchUp())}
          />
        ) : null}
        {problem ? <ErrorNotice title={t("notifications.refused")} description={problem} /> : null}
        <fieldset className="grid gap-2" disabled={!editable}>
          <legend className="mb-2 text-sm font-medium">{t("notifications.rules.states")}</legend>
          <div className="flex flex-wrap gap-x-6 gap-y-2">
            {ANNOUNCED_STATES.map((state) => (
              <label key={state} className="inline-flex items-center gap-2 text-sm">
                <Checkbox checked={draft.states.includes(state)} onCheckedChange={(checked) => toggle(state, checked === true)} />
                {t(`status.${state}`)}
              </label>
            ))}
          </div>
        </fieldset>
        <div className="flex items-center justify-between gap-4">
          <Label htmlFor="notifications-recovered">{t("notifications.rules.recovered")}</Label>
          <Switch id="notifications-recovered" disabled={!editable} checked={draft.recovered} onCheckedChange={(recovered) => setDraft((current) => ({ ...current, recovered }))} />
        </div>
        <Allowed area="notifications" action="update">
          <div>
            <Button type="button" onClick={() => void save()} disabled={change.isPending}>
              {t("notifications.rules.save")}
            </Button>
          </div>
        </Allowed>
      </div>
    </SectionCard>
  );
}
