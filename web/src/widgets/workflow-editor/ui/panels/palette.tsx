"use client";

import { cn } from "cn";
import { Search } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { KIND_GROUPS, type Kind, type Target } from "@/entities/workflow";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, Input } from "@/shared/ui/primitives";

import { kindPlaceable, slotContext } from "../../model/checks/placing";
import { useEditor } from "../../model/editor-context";
import { FALLBACK_ICON, GROUP_TONE, KIND_ICONS } from "../nodes/kind-style";

const QUICK_ENDS = [
  { key: "endRun", kind: "stop", loop: false },
  { key: "skip", kind: "nothing", loop: false },
  { key: "leaveLoop", kind: "break", loop: true },
  { key: "nextPass", kind: "continue", loop: true },
] as const;

export type PaletteProps = { target: Target | null; onChoose: (kind: Kind, target: Target) => void; onClose: () => void };

export function Palette({ target, onChoose, onClose }: PaletteProps) {
  const t = useTranslations("workflowEditor.palette");
  const help = useTranslations("workflowHelp");
  const common = useTranslations("common");
  const editor = useEditor();
  const [query, setQuery] = useState("");
  const text = (key: string) => (help.has(key as "kinds.if.name") ? help(key as "kinds.if.name") : "");
  const needle = query.trim().toLowerCase();
  const matches = (kind: Kind) =>
    needle === "" || [kind.name, text(`kinds.${kind.name}.name`), text(`kinds.${kind.name}.description`)].some((value) => value.toLowerCase().includes(needle));
  const shown = editor.catalogue.kinds.filter((kind) => matches(kind) && (target === null || kindPlaceable(kind.name, target)));
  const context = target ? slotContext(target) : { inBranch: false, inLoop: false };
  const quick = QUICK_ENDS.filter((entry) => context.inBranch && (!entry.loop || context.inLoop))
    .map((entry) => ({ ...entry, kind: editor.catalogue.kinds.find((kind) => kind.name === entry.kind) }))
    .filter((entry): entry is typeof entry & { kind: Kind } => entry.kind !== undefined);
  return (
    <Dialog
      open={target !== null}
      onOpenChange={(open) => {
        if (!open) {
          setQuery("");
          onClose();
        }
      }}
    >
      <DialogContent closeLabel={common("close")} className="max-h-[85dvh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <div className="relative">
          <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden />
          <Input autoFocus aria-label={t("search")} placeholder={t("search")} className="ps-9" value={query} onChange={(change) => setQuery(change.target.value)} />
        </div>
        {quick.length > 0 ? (
          <section className="grid gap-2" aria-label={t("quick.title")}>
            <h3 className="text-sm font-semibold">{t("quick.title")}</h3>
            <div className="flex flex-wrap gap-2">
              {quick.map((entry) => {
                const Icon = KIND_ICONS[entry.kind.name] ?? FALLBACK_ICON;
                return (
                  <button
                    key={entry.key}
                    type="button"
                    className="flex items-center gap-2 rounded-full border border-glass-edge bg-glass-tint px-3 py-1.5 text-sm transition-colors hover:border-primary hover:bg-accent"
                    onClick={() => {
                      setQuery("");
                      if (target) {
                        onChoose(entry.kind, target);
                      }
                    }}
                  >
                    <Icon className="size-4" aria-hidden />
                    {t(`quick.${entry.key}`)}
                  </button>
                );
              })}
            </div>
          </section>
        ) : null}
        {shown.length === 0 ? <p className="text-sm text-muted-foreground">{t("noMatches")}</p> : null}
        {KIND_GROUPS.map((group) => {
          const kinds = shown.filter((kind) => kind.group === group);
          if (kinds.length === 0) {
            return null;
          }
          return (
            <section key={group} className="grid gap-2" aria-label={text(`groups.${group}.name`)}>
              <div>
                <h3 className="text-sm font-semibold">{text(`groups.${group}.name`)}</h3>
                <p className="text-xs text-muted-foreground">{text(`groups.${group}.description`)}</p>
              </div>
              <div className="grid gap-2 sm:grid-cols-2">
                {kinds.map((kind) => {
                  const Icon = KIND_ICONS[kind.name] ?? FALLBACK_ICON;
                  return (
                    <button
                      key={kind.name}
                      type="button"
                      className="group grid grid-cols-[auto_1fr] items-start gap-x-3 gap-y-1 rounded-xl border border-glass-edge bg-glass-tint p-3 text-start transition-colors hover:border-primary hover:bg-accent"
                      onClick={() => {
                        setQuery("");
                        if (target) {
                          onChoose(kind, target);
                        }
                      }}
                    >
                      <span className={cn("row-span-3 grid size-9 place-items-center rounded-lg", GROUP_TONE[kind.group].badge)}>
                        <Icon className="size-4" aria-hidden />
                      </span>
                      <span className="text-sm font-medium">{text(`kinds.${kind.name}.name`) || kind.name}</span>
                      <span className="text-xs text-muted-foreground">{text(`kinds.${kind.name}.description`)}</span>
                      <span className="truncate font-mono text-[11px] text-muted-foreground/80">{text(`kinds.${kind.name}.example`)}</span>
                    </button>
                  );
                })}
              </div>
            </section>
          );
        })}
      </DialogContent>
    </Dialog>
  );
}
