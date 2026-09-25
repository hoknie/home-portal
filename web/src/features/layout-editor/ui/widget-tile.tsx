"use client";

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { GripVertical, Settings2, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import type { WidgetSize } from "@/shared/api";
import { cn } from "@/shared/lib/cn";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Badge, Button } from "@/shared/ui/primitives";

import { WIDGET_KIND } from "../model/drag";
import type { DraftWidget } from "../model/draft";
import { ResizeHandle } from "./resize-handle";

export type WidgetTileProps = {
  widget: DraftWidget;
  title: string;
  known: boolean;
  spanOf: (size: WidgetSize) => string;
  errors: string[];
  onResize: (size: WidgetSize) => void;
  onEdit: () => void;
  onRemove: () => void;
};

export function WidgetTile({ widget, title, known, spanOf, errors, onResize, onEdit, onRemove }: WidgetTileProps) {
  const t = useTranslations("layoutEditor");
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: widget.uid,
    data: { kind: WIDGET_KIND, section: widget.section },
  });
  const [removing, setRemoving] = useState(false);
  const [preview, setPreview] = useState<WidgetSize | null>(null);
  const element = useRef<HTMLDivElement | null>(null);
  const shown = preview ?? widget.size;
  return (
    <div
      ref={(node) => {
        element.current = node;
        setNodeRef(node);
      }}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      className={cn("relative min-w-0", spanOf(shown), isDragging && "z-10 opacity-70")}
      data-widget={widget.uid}
      data-size={widget.size}
    >
      <div className={cn("glass-panel grid gap-3 rounded-xl p-3", errors.length > 0 && "border-destructive", preview && "ring-2 ring-primary/60")}>
        <div className="flex items-center gap-2">
          <button
            type="button"
            className="flex size-8 cursor-grab items-center justify-center rounded-md text-muted-foreground hover:bg-glass-tint focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
            aria-label={t("move", { title })}
            {...attributes}
            {...listeners}
          >
            <GripVertical className="size-4" aria-hidden />
          </button>
          <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-medium">{title}</p>
            <p className="truncate font-mono text-xs text-muted-foreground">{widget.id ?? widget.type}</p>
          </div>
          {widget.public ? <Badge variant="outline">{t("public")}</Badge> : null}
        </div>
        {known ? null : <p className="text-xs text-muted-foreground">{t("unknownType", { type: widget.type })}</p>}
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="outline" data-size-label="">
            {t(`sizes.${shown}`)}
          </Badge>
          <div className="ml-auto flex gap-1">
            <Button type="button" variant="ghost" size="icon" aria-label={t("edit")} onClick={onEdit}>
              <Settings2 aria-hidden />
            </Button>
            <Button type="button" variant="ghost" size="icon" aria-label={t("remove")} onClick={() => setRemoving(true)}>
              <Trash2 aria-hidden />
            </Button>
          </div>
        </div>
        {errors.map((error) => (
          <p key={error} role="alert" className="text-xs text-destructive">
            {error}
          </p>
        ))}
      </div>
      <ResizeHandle
        title={title}
        size={widget.size}
        gridWidth={() => element.current?.parentElement?.getBoundingClientRect().width ?? 0}
        onPreview={setPreview}
        onResize={onResize}
      />
      <ConfirmDialog
        open={removing}
        title={t("removeTitle", { title })}
        description={t("removeDescription")}
        confirmLabel={t("remove")}
        onConfirm={() => {
          setRemoving(false);
          onRemove();
        }}
        onOpenChange={setRemoving}
      />
    </div>
  );
}
