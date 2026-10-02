"use client";

import { Settings2, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { type ReactNode, useRef, useState } from "react";

import { cn } from "@/shared/lib/cn";
import { GAP_PIXELS, cellClasses, cellStyle, positionOf, useAutoRows } from "@/shared/lib/widget-grid";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Badge, Button } from "@/shared/ui/kit";

import type { DraftWidget } from "../model/draft";
import type { Place, Step } from "../model/order";
import type { Size } from "../model/resize";
import { MoveHandle } from "./move-handle";
import { ResizeHandle, useSizeText } from "./resize-handle";

export type WidgetTileProps = {
  widget: DraftWidget;
  title: string;
  type: string;
  known: boolean;
  errors: string[];
  content: ReactNode;
  onResize: (size: Size) => void;
  placeholder?: boolean;
  sections: () => string[];
  onMove: (place: Place) => void;
  onStep: (step: Step) => void;
  onPreviewMove: (place: Place | null) => void;
  onConfigure: () => void;
  onRemove: () => void;
};

const TOOL = "flex size-8 items-center justify-center rounded-md border border-glass-edge bg-background text-muted-foreground hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none";

export function WidgetTile({ widget, title, type, known, errors, content, placeholder = false, sections, onResize, onMove, onStep, onPreviewMove, onConfigure, onRemove }: WidgetTileProps) {
  const t = useTranslations("layoutEditor");
  const text = useSizeText();
  const [removing, setRemoving] = useState(false);
  const [preview, setPreview] = useState<Size | null>(null);
  const element = useRef<HTMLDivElement | null>(null);
  const size: Size = { width: widget.width, height: widget.height };
  const shown = preview ?? size;
  const [body, rows] = useAutoRows<HTMLDivElement>(shown.height === "auto", widget.key ?? widget.uid);
  const position = positionOf(widget);
  const measure = () => ({
    gridWidth: element.current?.parentElement?.getBoundingClientRect().width ?? 0,
    startPixels: Math.max(0, (element.current?.getBoundingClientRect().height ?? GAP_PIXELS) - GAP_PIXELS),
  });
  return (
    <div
      ref={element}
      style={cellStyle(shown.width, shown.height, rows, position)}
      className={cn("group/tile relative", cellClasses(position))}
      data-widget={widget.uid}
      data-width={widget.width}
      data-height={widget.height}
      data-column={widget.column ?? undefined}
      data-row={widget.row ?? undefined}
      data-placeholder={placeholder || undefined}
    >
      <div
        className={cn(
          "relative h-full rounded-2xl outline-2 outline-offset-2 outline-transparent transition-[outline-color] group-hover/tile:outline-primary/30 group-focus-within/tile:outline-primary/50",
          errors.length > 0 && "outline-destructive",
          preview && "outline-primary",
          placeholder && "outline-dashed outline-primary",
        )}
      >
        <div ref={body} className={cn(shown.height !== "auto" && "h-full overflow-hidden")}>
          <div inert className={cn("pointer-events-none h-full select-none", placeholder && "opacity-40")} data-tile-content="">
            {known ? content : <UnknownTile label={t("unknownType", { type })} />}
          </div>
        </div>
        <div className="absolute top-2 right-2 flex items-center gap-1 opacity-0 transition-opacity group-hover/tile:opacity-100 group-focus-within/tile:opacity-100 max-sm:opacity-100">
          <Button type="button" variant="ghost" size="icon" className={TOOL} aria-label={t("editIn", { title })} title={t("editIn", { title })} onClick={onConfigure}>
            <Settings2 aria-hidden />
          </Button>
          <Button type="button" variant="ghost" size="icon" className={TOOL} aria-label={t("remove")} title={t("remove")} onClick={() => setRemoving(true)}>
            <Trash2 aria-hidden />
          </Button>
        </div>
        {preview ? (
          <Badge className="absolute right-2 bottom-2 tabular-nums" data-size-label="">
            {text.both(preview)}
          </Badge>
        ) : null}
      </div>
      {errors.length > 0 ? (
        <div className="absolute inset-x-2 bottom-6 z-10 grid gap-1" data-tile-errors="">
          {errors.map((error) => (
            <p key={error} role="alert" className="rounded-md bg-background px-2 py-1 text-xs text-destructive">
              {error}
            </p>
          ))}
        </div>
      ) : null}
      <MoveHandle title={title} uid={widget.uid} section={widget.section} sections={sections} tile={() => element.current} onPreview={onPreviewMove} onMove={onMove} onStep={onStep} />
      {(["width", "height", "both"] as const).map((axis) => (
        <ResizeHandle key={axis} axis={axis} title={title} size={size} measure={measure} onPreview={setPreview} onResize={onResize} />
      ))}
      <ConfirmDialog
        open={removing}
        title={t("removeTitle", { title })}
        description={t("removeFromPage")}
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

function UnknownTile({ label }: { label: string }) {
  return <div className="grid h-full min-h-20 place-items-center rounded-2xl border border-dashed p-4 text-center text-sm text-muted-foreground">{label}</div>;
}
