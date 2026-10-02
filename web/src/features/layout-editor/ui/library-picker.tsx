"use client";

import { Puzzle } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { type LibraryWidget, useLibrary } from "@/entities/dashboard";
import { routes } from "@/shared/config";
import { BareButton, Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, Input } from "@/shared/ui/kit";

import type { KindLabel } from "../model/catalog";

export type LibraryPickerProps = { open: boolean; kinds: KindLabel[]; onChoose: (widget: LibraryWidget) => void; onClose: () => void };

export function LibraryPicker({ open, kinds, onChoose, onClose }: LibraryPickerProps) {
  const t = useTranslations("layoutEditor.picker");
  const widgets = useLibrary().data?.data.widgets ?? [];
  const [query, setQuery] = useState("");
  const kindOf = (type: string) => kinds.find((kind) => kind.type === type);
  const nameOf = (widget: LibraryWidget) => widget.title ?? kindOf(widget.type)?.title ?? widget.id;
  const wanted = query.trim().toLowerCase();
  const shown = widgets.filter((widget) => wanted === "" || `${nameOf(widget)} ${widget.id} ${kindOf(widget.type)?.title ?? widget.type}`.toLowerCase().includes(wanted));
  return (
    <Dialog open={open} onOpenChange={(next) => (next ? undefined : onClose())}>
      <DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <Input type="search" aria-label={t("search")} placeholder={t("search")} value={query} onChange={(event) => setQuery(event.target.value)} autoFocus />
        {shown.length === 0 ? <p className="text-sm text-muted-foreground">{widgets.length === 0 ? t("empty") : t("nothing")}</p> : null}
        <ul className="grid gap-2 sm:grid-cols-2">
          {shown.map((widget) => {
            const Icon = kindOf(widget.type)?.icon ?? Puzzle;
            return (
              <li key={widget.id}>
                <BareButton
                  onClick={() => onChoose(widget)}
                  className="flex h-full w-full items-start gap-3 rounded-xl border p-3 text-left transition-colors outline-none hover:border-primary hover:bg-primary/5 focus-visible:ring-[3px] focus-visible:ring-ring/50"
                  data-library-choice={widget.id}
                >
                  <Icon className="mt-0.5 size-5 shrink-0 text-primary" aria-hidden />
                  <span className="grid gap-0.5">
                    <span className="text-sm font-medium">{nameOf(widget)}</span>
                    <span className="text-xs text-muted-foreground">{t("meta", { type: kindOf(widget.type)?.title ?? widget.type, placed: widget.placed })}</span>
                  </span>
                </BareButton>
              </li>
            );
          })}
        </ul>
        <Link href={routes.adminLibrary} className="text-sm text-primary underline underline-offset-2">
          {t("openLibrary")}
        </Link>
      </DialogContent>
    </Dialog>
  );
}
