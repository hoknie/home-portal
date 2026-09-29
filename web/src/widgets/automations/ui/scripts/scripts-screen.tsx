"use client";

import { FileCode } from "lucide-react";
import { useTranslations } from "next-intl";
import { useCallback, useState } from "react";
import { toast } from "sonner";

import { useAutomations, useScripts } from "@/entities/automation";
import { type ScriptEntry, useCreateFolder, useCreateScript, useDeleteFolder, useDeleteScript, useMoveScript, useScriptTree } from "@/entities/script";
import { useWebhooks } from "@/entities/webhook";
import { useWorkflows } from "@/entities/workflow";
import { RequestError, ValidationError } from "@/shared/api";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { useLeaveGuard } from "@/shared/lib/leave-guard";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { PageHeader } from "@/shared/ui/page-header";
import { Button, Skeleton } from "@/shared/ui/primitives";

import { NEW_SCRIPT, usersOf } from "../../model/script-usage";
import { PlaceDialog } from "./script-dialogs";
import { ScriptEditor } from "./script-editor";
import { ScriptTreePane } from "./script-tree";

const SETTING = "[scripts]\nediting = true";

type Asking = { kind: "script" } | { kind: "folder" } | { kind: "rename"; entry: ScriptEntry } | { kind: "delete"; entry: ScriptEntry } | null;

function messageOf(error: unknown) {
  if (error instanceof ValidationError) {
    return error.fields.map((field) => field.message).join("; ");
  }
  return error instanceof RequestError || error instanceof Error ? error.message : String(error);
}

export function ScriptsScreen() {
  const t = useTranslations("scripts");
  const trail = useTrail();
  const listing = useScripts();
  const editing = listing.data?.editing ?? false;
  const tree = useScriptTree({ enabled: editing });
  const automations = useAutomations();
  const webhooks = useWebhooks();
  const workflows = useWorkflows();
  const createScript = useCreateScript();
  const createFolder = useCreateFolder();
  const moveScript = useMoveScript();
  const deleteScript = useDeleteScript();
  const deleteFolder = useDeleteFolder();
  const [selected, setSelected] = useState<string | null>(null);
  const [dirty, setDirty] = useState(false);
  const [asking, setAsking] = useState<Asking>(null);
  const [problem, setProblem] = useState<string | null>(null);
  useLeaveGuard(dirty, t("leave"));
  const onDirty = useCallback((next: boolean) => setDirty(next), []);
  const data = tree.data;
  const readOnly = !(data?.inside ?? false);
  const entry = data?.files.find((file) => file.path === selected) ?? null;
  const select = (path: string) => {
    if (path !== selected && (!dirty || window.confirm(t("leave")))) {
      setDirty(false);
      setSelected(path);
    }
  };
  const ask = (next: Asking) => {
    setProblem(null);
    setAsking(next);
  };
  const attempt = async (work: () => Promise<unknown>, after?: () => void) => {
    try {
      await work();
      setAsking(null);
      after?.();
    } catch (error) {
      setProblem(messageOf(error));
    }
  };
  const users =
    entry || asking?.kind === "delete"
      ? {
          automations: automations.data?.data.automations ?? [],
          webhooks: webhooks.data?.data.webhooks ?? [],
          workflows: workflows.data?.data.workflows ?? [],
        }
      : null;
  const deleting = asking?.kind === "delete" ? asking.entry : null;
  const usedBy = deleting && users ? usersOf(deleting.path, users) : [];
  const header = <PageHeader breadcrumbs={trail.of(trail.section("scripts"))} title={t("title")} description={t("subtitle")} />;
  if (listing.error && !listing.data) {
    return (
      <div className="grid gap-8">
        {header}
        <ErrorNotice title={t("title")} description={listing.error.message} onRetry={() => void listing.refetch()} />
      </div>
    );
  }
  if (!listing.data) {
    return (
      <div className="grid gap-8">
        {header}
        <Skeleton className="h-64 w-full" aria-busy="true" />
      </div>
    );
  }
  if (!editing) {
    return (
      <div className="grid gap-8">
        {header}
        <p role="note" className="rounded-lg border border-glass-edge p-4 text-sm">
          {t("off")}
          <code className="mt-2 block rounded-md border border-glass-edge bg-glass-tint px-2 py-1 font-mono text-xs whitespace-pre">{SETTING}</code>
        </p>
      </div>
    );
  }
  return (
    <div className="grid gap-8">
      {header}
      {tree.error && !data ? <ErrorNotice title={t("title")} description={tree.error.message} onRetry={() => void tree.refetch()} /> : null}
      {data && readOnly ? (
        <p role="note" className="rounded-lg border border-glass-edge p-3 text-sm text-muted-foreground">
          {t("readOnly")}
        </p>
      ) : null}
      {data && !data.exists ? (
        <div role="note" className="grid justify-items-start gap-3 rounded-lg border border-glass-edge p-4 text-sm">
          <p>{t("missing", { directory: data.directory })}</p>
          {readOnly ? null : (
            <Button type="button" size="sm" onClick={() => void attempt(() => createFolder.mutateAsync(null))}>
              {t("createDirectory")}
            </Button>
          )}
        </div>
      ) : null}
      {data && data.exists ? (
        <div className="grid items-start gap-6 lg:grid-cols-[19rem_minmax(0,1fr)]">
          <ScriptTreePane
            tree={data}
            selected={selected}
            readOnly={readOnly}
            onSelect={select}
            onNewScript={() => ask({ kind: "script" })}
            onNewFolder={() => ask({ kind: "folder" })}
            onRename={(chosen) => ask({ kind: "rename", entry: chosen })}
            onDelete={(chosen) => ask({ kind: "delete", entry: chosen })}
            onDeleteFolder={(name) => void deleteFolder.mutateAsync(name).catch((error: unknown) => toast.error(messageOf(error)))}
          />
          <div className="min-w-0">
            {entry ? (
              <ScriptEditor key={entry.path} entry={entry} userId={data.user_id} directory={data.directory} readOnly={readOnly} onDirty={onDirty} />
            ) : (
              <EmptyState icon={FileCode} title={t("chooseTitle")} description={t("choose")} />
            )}
          </div>
        </div>
      ) : null}
      {data ? (
        <>
          <PlaceDialog
            open={asking?.kind === "script"}
            title={t("newScriptTitle")}
            confirmLabel={t("create")}
            folders={data.folders}
            folder={entry?.folder ?? ""}
            error={problem}
            pending={createScript.isPending}
            onOpenChange={(open) => ask(open ? { kind: "script" } : null)}
            onConfirm={(path) =>
              void attempt(
                () => createScript.mutateAsync({ path, content: NEW_SCRIPT }),
                () => setSelected(path),
              )
            }
          />
          <PlaceDialog
            open={asking?.kind === "folder"}
            title={t("newFolderTitle")}
            confirmLabel={t("create")}
            folders={[]}
            withFolder={false}
            error={problem}
            pending={createFolder.isPending}
            onOpenChange={(open) => ask(open ? { kind: "folder" } : null)}
            onConfirm={(name) => void attempt(() => createFolder.mutateAsync(name))}
          />
          <PlaceDialog
            open={asking?.kind === "rename"}
            title={t("renameTitle", {
              path: asking?.kind === "rename" ? asking.entry.path : "",
            })}
            confirmLabel={t("move")}
            folders={data.folders}
            folder={asking?.kind === "rename" ? (asking.entry.folder ?? "") : ""}
            name={asking?.kind === "rename" ? asking.entry.name : ""}
            error={problem}
            pending={moveScript.isPending}
            onOpenChange={(open) => (open ? null : ask(null))}
            onConfirm={(to) => {
              if (asking?.kind === "rename") {
                const from = asking.entry;
                void attempt(
                  () =>
                    moveScript.mutateAsync({
                      from: from.path,
                      to,
                      revision: from.revision ?? "",
                    }),
                  () => selected === from.path && setSelected(to),
                );
              }
            }}
          />
          <ConfirmDialog
            open={deleting !== null}
            title={t("deleteTitle", { path: deleting?.path ?? "" })}
            description={[t("deleteDescription"), usedBy.length > 0 ? t("deleteUsed", { users: usedBy.join(", ") }) : "", problem ?? ""]
              .filter(Boolean)
              .join(" ")}
            confirmLabel={t("delete")}
            pending={deleteScript.isPending}
            onOpenChange={(open) => (open ? null : ask(null))}
            onConfirm={() => {
              if (deleting) {
                void attempt(
                  () =>
                    deleteScript.mutateAsync({
                      path: deleting.path,
                      revision: deleting.revision ?? "",
                    }),
                  () => selected === deleting.path && setSelected(null),
                );
              }
            }}
          />
        </>
      ) : null}
    </div>
  );
}
