"use client";

import { useDroppable } from "@dnd-kit/core";
import { SortableContext, rectSortingStrategy, useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVertical, Plus, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { type ReactNode, useState } from "react";

import type { Section } from "@/shared/api";
import { cn } from "@/shared/lib/cn";
import { Button, Input } from "@/shared/ui/primitives";

import type { WidgetKind } from "../model/catalog";
import { AREA_KIND, SECTION_KIND, SECTION_PREFIX, SECTION_SORT_PREFIX } from "../model/drag";

export const SELECT =
  "h-8 rounded-md border border-input bg-glass-tint px-2 text-xs shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50";

export type SectionBlockProps = {
  section: Section;
  count: number;
  widgetIds: string[];
  kinds: WidgetKind[];
  errors: string[];
  children: ReactNode;
  onRename: (title: string) => void;
  onRemove: () => void;
  onAdd: (type: string) => void;
};

export function SectionBlock(props: SectionBlockProps) {
  const t = useTranslations("layoutEditor");
  const { section, count, widgetIds, kinds } = props;
  const { setNodeRef: setAreaRef } = useDroppable({ id: `${SECTION_PREFIX}${section.id}`, data: { kind: AREA_KIND, section: section.id } });
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: `${SECTION_SORT_PREFIX}${section.id}`,
    data: { kind: SECTION_KIND, section: section.id },
  });
  const [type, setType] = useState(kinds[0]?.type ?? "");
  const title = section.title ?? t("untitled");
  return (
    <section
      ref={setNodeRef}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      className={cn("grid gap-3 rounded-2xl border border-dashed border-glass-edge p-3", isDragging && "z-20 opacity-70")}
      data-section={section.id}
      data-section-block=""
    >
      <div className="flex flex-wrap items-center gap-2">
        <button
          type="button"
          className="flex size-8 cursor-grab items-center justify-center rounded-md text-muted-foreground hover:bg-glass-tint focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
          aria-label={t("moveSection", { title })}
          {...attributes}
          {...listeners}
        >
          <GripVertical className="size-4" aria-hidden />
        </button>
        <Input
          aria-label={t("sectionTitle")}
          placeholder={t("untitled")}
          className="h-8 max-w-xs"
          value={section.title ?? ""}
          onChange={(event) => props.onRename(event.target.value)}
        />
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label={t("removeSection")}
          disabled={widgetIds.length > 0 || count <= 1}
          onClick={props.onRemove}
        >
          <Trash2 aria-hidden />
        </Button>
        <div className="ml-auto flex items-center gap-2">
          <select aria-label={t("chooseType")} className={SELECT} value={type} onChange={(event) => setType(event.target.value)}>
            {kinds.map((kind) => (
              <option key={kind.type} value={kind.type}>
                {kind.title}
              </option>
            ))}
          </select>
          <Button type="button" variant="outline" size="sm" onClick={() => props.onAdd(type)}>
            <Plus aria-hidden />
            {t("addWidget")}
          </Button>
        </div>
      </div>
      {props.errors.map((error) => (
        <p key={error} role="alert" className="text-xs text-destructive">
          {error}
        </p>
      ))}
      <SortableContext id={section.id} items={widgetIds} strategy={rectSortingStrategy}>
        <div ref={setAreaRef} className="grid min-h-16 grid-cols-12 gap-3">
          {props.children}
          {widgetIds.length === 0 ? (
            <p className="col-span-12 self-center py-4 text-center text-sm text-muted-foreground">{t("emptySection")}</p>
          ) : null}
        </div>
      </SortableContext>
    </section>
  );
}
