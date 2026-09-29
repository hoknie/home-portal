"use client";

import { useQueryClient } from "@tanstack/react-query";
import { useTranslations } from "next-intl";
import { Save } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { fixOf, problemOf } from "@/entities/automation";
import { type ScriptEntry, type ScriptText, fetchScriptText, parseHeader, scriptKey, useSaveScript, useScriptText } from "@/entities/script";
import { ConflictError } from "@/shared/api";
import { CodeArea } from "@/shared/ui/code-area";
import { RelativeTime } from "@/shared/ui/relative-time";
import { SectionCard } from "@/shared/ui/section-card";
import { Badge, Button, Skeleton } from "@/shared/ui/primitives";

import { DeclaredSummary } from "./declared-summary";
import { HeaderHelp, withExampleHeader } from "./header-help";

export type ScriptEditorProps = {
  entry: ScriptEntry;
  userId: number;
  directory: string;
  readOnly: boolean;
  onDirty: (dirty: boolean) => void;
};

function Status({ entry }: { entry: ScriptEntry }) {
  const t = useTranslations("scripts");
  return (
    <Badge variant="outline" className={entry.runnable ? "border-status-up/50 text-status-up" : "border-status-degraded/50 text-status-degraded"}>
      <span className={entry.runnable ? "size-1.5 rounded-full bg-status-up" : "size-1.5 rounded-full bg-status-degraded"} aria-hidden />
      {entry.runnable ? t("runnable") : t("notRunnable")}
    </Badge>
  );
}

function Details({ entry }: { entry: ScriptEntry }) {
  const t = useTranslations("scripts");
  const size = entry.size < 1024 ? t("sizeBytes", { size: entry.size }) : t("sizeKib", { size: Math.round((entry.size / 1024) * 10) / 10 });
  return (
    <dl className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground">
      <div className="flex gap-1">
        <dt className="sr-only">{t("mode", { mode: "" })}</dt>
        <dd className="font-mono">{t("mode", { mode: entry.mode })}</dd>
      </div>
      <div className="flex gap-1">
        <dd className="font-mono">{size}</dd>
      </div>
      {entry.modified ? (
        <div className="flex gap-1">
          <dt>{t("modified")}</dt>
          <dd>
            <RelativeTime moment={entry.modified} />
          </dd>
        </div>
      ) : null}
    </dl>
  );
}

function Problem({ entry, userId }: { entry: ScriptEntry; userId: number }) {
  const help = useTranslations("scriptHelp");
  if (entry.runnable) {
    return null;
  }
  const problem = problemOf(entry.code);
  const concerns = entry.concerns ?? entry.path;
  const fix = problem ? fixOf(problem, concerns, userId) : null;
  return (
    <div role="note" className="grid gap-1.5 rounded-lg border border-status-degraded/40 bg-status-degraded/5 p-3 text-sm">
      <p>{problem ? help(`problems.${problem}`, { path: concerns }) : entry.problem}</p>
      {fix ? <code className="rounded-md border border-glass-edge bg-glass-tint px-2 py-1 font-mono text-xs break-all">{fix}</code> : null}
    </div>
  );
}

function title(entry: ScriptEntry, directory: string) {
  return entry.folder ? `${directory}/${entry.folder}` : directory;
}

type TextEditorProps = ScriptEditorProps & { loaded: ScriptText };

function TextEditor({ entry, userId, directory, readOnly, onDirty, loaded }: TextEditorProps) {
  const t = useTranslations("scripts");
  const root = useTranslations();
  const client = useQueryClient();
  const save = useSaveScript();
  const [text, setText] = useState(loaded.content);
  const [base, setBase] = useState(loaded.content);
  const [revision, setRevision] = useState(loaded.revision);
  const [conflict, setConflict] = useState(false);
  const dirty = text !== base;
  useEffect(() => onDirty(dirty), [dirty, onDirty]);
  const header = useMemo(() => parseHeader(text), [text]);
  const store = async (content: string, at: string) => {
    const saved = await save.mutateAsync({ path: entry.path, content, revision: at });
    const next = (saved.revision ?? "").replace(/^W\//, "").replaceAll('"', "");
    setRevision(next);
    setBase(content);
    client.setQueryData(scriptKey(entry.path), { path: entry.path, content, revision: next, entry: saved.data });
    setConflict(false);
    toast.success(t("saved"));
  };
  const saving = async () => {
    if (readOnly) {
      return;
    }
    try {
      await store(text, revision);
    } catch (error) {
      if (error instanceof ConflictError) {
        setConflict(true);
      } else {
        toast.error(error instanceof Error ? error.message : root("errors.generic"));
      }
    }
  };
  const reload = async () => {
    const fresh = await fetchScriptText(entry.path);
    client.setQueryData(scriptKey(entry.path), fresh);
    setText(fresh.content);
    setBase(fresh.content);
    setRevision(fresh.revision);
    setConflict(false);
  };
  const overwrite = async () => {
    const fresh = await fetchScriptText(entry.path);
    await store(text, fresh.revision);
  };
  const actions = readOnly ? null : (
    <>
      {dirty ? (
        <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
          <span className="size-1.5 rounded-full bg-primary" aria-hidden />
          {t("unsaved")}
        </span>
      ) : null}
      <Button type="button" size="sm" disabled={!dirty || save.isPending} onClick={() => void saving()} title={t("shortcut")}>
        <Save aria-hidden />
        {t("save")}
      </Button>
    </>
  );
  return (
    <div className="grid gap-6 2xl:grid-cols-[minmax(0,1fr)_22rem]">
      <SectionCard title={entry.name} description={title(entry, directory)} badge={<Status entry={entry} />} actions={actions}>
        <div className="grid gap-3">
          <Details entry={entry} />
          <Problem entry={entry} userId={userId} />
          {conflict ? (
            <div role="alert" className="flex flex-wrap items-center gap-2 rounded-lg border border-destructive/40 p-3 text-sm">
              <span>{t("conflict")}</span>
              <Button type="button" size="sm" variant="outline" onClick={() => void reload()}>
                {t("reload")}
              </Button>
              <Button type="button" size="sm" onClick={() => void overwrite()}>
                {t("overwrite")}
              </Button>
            </div>
          ) : null}
          <CodeArea aria-label={t("text")} value={text} onChange={setText} onSave={() => void saving()} readOnly={readOnly} className="min-h-[28rem]" />
        </div>
      </SectionCard>
      <div className="grid content-start gap-6">
        <section aria-label={t("declared")}>
          <SectionCard title={t("declared")}>
            <DeclaredSummary name={entry.name} header={header} />
          </SectionCard>
        </section>
        <SectionCard>
          <HeaderHelp onInsert={readOnly ? undefined : () => setText(withExampleHeader(text))} />
        </SectionCard>
      </div>
    </div>
  );
}

export function ScriptEditor(props: ScriptEditorProps) {
  const { entry, userId, directory } = props;
  const t = useTranslations("scripts");
  const loaded = useScriptText(entry.text ? entry.path : null);
  if (!entry.text) {
    return (
      <SectionCard title={entry.name} description={title(entry, directory)} badge={<Status entry={entry} />}>
        <div className="grid gap-3">
          <Details entry={entry} />
          <Problem entry={entry} userId={userId} />
          <p role="note" className="text-sm text-muted-foreground">
            {t(`unreadable.${entry.unreadable ?? "binary"}`)}
          </p>
        </div>
      </SectionCard>
    );
  }
  return loaded.data ? <TextEditor {...props} loaded={loaded.data} /> : <Skeleton className="h-96 w-full rounded-xl" aria-busy="true" />;
}
