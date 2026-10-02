"use client";

import { FileCode, Folder, FolderPlus, MoreHorizontal, Plus, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";

import { useCan } from "@/entities/session";
import type { ScriptEntry, ScriptTree } from "@/entities/script";
import { Badge, BareButton, Button, DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/shared/ui/kit";
import { SectionCard } from "@/shared/ui/section-card";

export type ScriptTreePaneProps = {
  tree: ScriptTree;
  selected: string | null;
  readOnly: boolean;
  onSelect: (path: string) => void;
  onNewScript: () => void;
  onNewFolder: () => void;
  onRename: (entry: ScriptEntry) => void;
  onDelete: (entry: ScriptEntry) => void;
  onDeleteFolder: (name: string) => void;
};

function FileRow({
  entry,
  selected,
  readOnly,
  onSelect,
  onRename,
  onDelete,
}: { entry: ScriptEntry } & Pick<ScriptTreePaneProps, "selected" | "readOnly" | "onSelect" | "onRename" | "onDelete">) {
  const t = useTranslations("scripts");
  const can = useCan();
  return (
    <li className="flex items-center gap-1">
      <BareButton
        aria-current={selected === entry.path ? "true" : undefined}
        onClick={() => onSelect(entry.path)}
        className="flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent aria-[current=true]:bg-accent aria-[current=true]:font-medium"
      >
        <FileCode className="size-4 shrink-0 text-palette-blue" aria-hidden />
        <span className="min-w-0 truncate font-mono">{entry.name}</span>
        {entry.runnable ? (
          <span className="ml-auto size-1.5 shrink-0 rounded-full bg-status-up" title={t("runnable")} aria-hidden />
        ) : (
          <Badge variant="outline" className="ml-auto shrink-0 border-status-degraded/50 text-xs text-status-degraded">
            {t("notRunnable")}
          </Badge>
        )}
      </BareButton>
      {readOnly || !(can("scripts", "update") || can("scripts", "delete")) ? null : (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button type="button" variant="ghost" size="icon" aria-label={`${entry.path}: ${t("rename")}, ${t("delete")}`}>
              <MoreHorizontal aria-hidden />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem disabled={entry.revision === null || !can("scripts", "update")} onSelect={() => onRename(entry)}>
              {t("rename")}
            </DropdownMenuItem>
            <DropdownMenuItem disabled={entry.revision === null || !can("scripts", "delete")} onSelect={() => onDelete(entry)}>
              {t("delete")}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      )}
    </li>
  );
}

export function ScriptTreePane(props: ScriptTreePaneProps) {
  const { tree, readOnly, onNewScript, onNewFolder, onDeleteFolder } = props;
  const t = useTranslations("scripts");
  const can = useCan();
  const top = tree.files.filter((entry) => entry.folder === null);
  const actions = readOnly || !can("scripts", "create") ? null : (
    <div className="flex gap-2">
      <Button type="button" size="sm" className="flex-1" onClick={onNewScript}>
        <Plus aria-hidden />
        {t("newScript")}
      </Button>
      <Button type="button" size="icon" variant="outline" className="size-8 shrink-0" aria-label={t("newFolder")} title={t("newFolder")} onClick={onNewFolder}>
        <FolderPlus aria-hidden />
      </Button>
    </div>
  );
  return (
    <nav aria-label={t("tree")}>
      <SectionCard
        title={t("tree")}
        description={t("filesCount", { count: tree.files.length })}
      >
        <div className="grid gap-3">
          {actions}
          {tree.files.length === 0 && tree.folders.length === 0 ? <p className="text-sm text-muted-foreground">{t("empty")}</p> : null}
          <ul className="grid gap-0.5">
            {tree.folders.map((folder) => {
              const inside = tree.files.filter((entry) => entry.folder === folder);
              return (
                <li key={folder}>
                  <details open className="group">
                    <summary className="flex cursor-pointer list-none items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-accent">
                      <Folder className="size-4 text-palette-amber" aria-hidden />
                      <span className="font-mono font-medium">{folder}</span>
                      <span className="text-xs text-muted-foreground">{inside.length}</span>
                      {!readOnly && can("scripts", "delete") && inside.length === 0 ? (
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          className="ml-auto size-7"
                          aria-label={t("deleteFolder", { name: folder })}
                          onClick={() => onDeleteFolder(folder)}
                        >
                          <Trash2 aria-hidden />
                        </Button>
                      ) : null}
                    </summary>
                    <ul className="ml-3.5 grid gap-0.5 border-l border-glass-edge pl-2">
                      {inside.map((entry) => (
                        <FileRow key={entry.path} entry={entry} {...props} />
                      ))}
                    </ul>
                  </details>
                </li>
              );
            })}
            {top.map((entry) => (
              <FileRow key={entry.path} entry={entry} {...props} />
            ))}
          </ul>
          {tree.left_out > 0 ? <p className="text-xs text-muted-foreground">{t("leftOut", { count: tree.left_out })}</p> : null}
          <p className="truncate font-mono text-xs text-muted-foreground" title={tree.directory}>
            {tree.directory}
          </p>
        </div>
      </SectionCard>
    </nav>
  );
}
