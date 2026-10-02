"use client";

import { DndContext, type DragEndEvent, KeyboardSensor, PointerSensor, closestCenter, useSensor, useSensors } from "@dnd-kit/core";
import { SortableContext, sortableKeyboardCoordinates, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { Library, Plus, Redo2, Undo2 } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";
import { type ReactNode, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { type Dashboard, type LibraryWidget, fetchLayout, useLibrary, useSaveLayout } from "@/entities/dashboard";
import { ConflictError, type Revisioned, ValidationError } from "@/shared/api";
import { routes } from "@/shared/config";
import { useDraftHistory } from "@/shared/lib/history";
import { useLeaveGuard } from "@/shared/lib/leave-guard";
import { ConflictNotice } from "@/shared/ui/conflict-notice";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { Button } from "@/shared/ui/primitives";

import type { KindLabel } from "../model/catalog";
import { SECTION_SORT_PREFIX, afterDrop } from "../model/drag";
import {
  type DraftWidget,
  addSection,
  fromLayout,
  placeWidget,
  removeSection,
  removeWidget,
  renameSection,
  sameDraft,
  toRequest,
  updateSection,
  updateWidget,
  widgetsOf,
} from "../model/draft";
import { type PlacedErrors, placeErrors } from "../model/errors";
import { type Target, cellsIn } from "../model/measure";
import { movedTo } from "../model/positions";
import type { Size } from "../model/resize";
import { dropDraft, keepDraft, keptDraft } from "../model/kept-draft";
import { useAnnouncements } from "../model/use-announcements";
import { LibraryPicker } from "./library-picker";
import type { Step } from "./move-handle";
import { useSizeText } from "./resize-handle";
import { SectionBlock } from "./section-block";
import { SectionDialog } from "./section-dialog";
import { WidgetTile } from "./widget-tile";

export type EditorProps = {
  loaded: Revisioned<Dashboard>;
  kinds: KindLabel[];
  renderWidget: (widget: LibraryWidget, placement: DraftWidget) => ReactNode;
  renderSettings: (id: string, close: () => void, adopt: (revision: string | null) => void) => ReactNode;
  builds?: (type: string) => boolean;
  onBuild?: (id: string, place: string) => void;
  onReload: () => void;
};

export function Editor({ loaded, kinds, renderWidget, renderSettings, builds, onBuild, onReload }: EditorProps) {
  const t = useTranslations("layoutEditor");
  const base = useMemo(() => fromLayout(loaded.data), [loaded]);
  const [notice, setNotice] = useState("");
  const sizeText = useSizeText();
  const [kept] = useState(() => keptDraft(base));
  useEffect(() => dropDraft(), []);
  const history = useDraftHistory(kept ?? base, sameDraft, (kind, label) => setNotice(t(`announce.${kind}`, { change: label })));
  const draft = history.draft;
  const library = useLibrary().data?.data.widgets ?? [];
  const [revision, setRevision] = useState(loaded.revision);
  const [configuring, setConfiguring] = useState<string | null>(null);
  const [picking, setPicking] = useState<string | null>(null);
  const [sectionEditing, setSectionEditing] = useState<string | null>(null);
  const [ghost, setGhost] = useState<(Target & { uid: string }) | null>(null);
  const [conflict, setConflict] = useState(false);
  const [errors, setErrors] = useState<PlacedErrors | null>(null);
  const save = useSaveLayout();
  const dirty = !sameDraft(draft, base);
  useLeaveGuard(dirty, t("leave"));
  const entryOf = (widget: DraftWidget) => library.find((entry) => entry.id === widget.widget) ?? null;
  const kindOf = (type: string) => kinds.find((kind) => kind.type === type) ?? null;
  const titleOf = (uid: string) => {
    const widget = draft.widgets.find((candidate) => candidate.uid === uid);
    const entry = widget ? entryOf(widget) : null;
    return entry ? (entry.title ?? kindOf(entry.type)?.title ?? entry.id) : (widget?.widget ?? uid);
  };
  const announcements = useAnnouncements(draft);
  const sensors = useSensors(useSensor(PointerSensor), useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }));

  const dropped = ({ active, over }: DragEndEvent) => {
    const item = (entry: { id: string | number; data: { current?: Record<string, unknown> } }) => ({
      id: String(entry.id),
      kind: (entry.data.current?.kind as string | undefined) ?? null,
      section: (entry.data.current?.section as string | undefined) ?? null,
    });
    history.change(t("steps.arranged"), (current) => afterDrop(current, item(active), over ? item(over) : null));
  };

  const resize = (uid: string, size: Size) => {
    const before = draft.widgets.find((widget) => widget.uid === uid);
    const title = titleOf(uid);
    history.change(t("steps.resized", { title }), (current) => updateWidget(current, uid, size));
    const widthChanged = before?.width !== size.width;
    const heightChanged = before?.height !== size.height;
    const announced = widthChanged && heightChanged ? "resizedBoth" : heightChanged ? "resizedHeight" : "resized";
    setNotice(t(`announce.${announced}`, { title, size: sizeText[announced === "resizedBoth" ? "both" : announced === "resizedHeight" ? "height" : "width"](size) }));
  };

  const moveTo = (uid: string, target: Target) => {
    const title = titleOf(uid);
    const measured = { ...cellsIn(target.section), ...cellsIn(draft.widgets.find((widget) => widget.uid === uid)?.section ?? target.section) };
    history.change(t("steps.moved", { title }), (current) => movedTo(current, uid, target.section, target, measured));
    const section = draft.sections.find((candidate) => candidate.id === target.section);
    setNotice(t("announce.moved", { title, column: target.column, row: target.row, section: section?.title ?? t("untitled") }));
  };

  const step = (uid: string, stepped: Step) => {
    const widget = draft.widgets.find((candidate) => candidate.uid === uid);
    if (!widget) {
      return;
    }
    const here = cellsIn(widget.section)[uid] ?? { column: widget.column ?? 1, row: widget.row ?? 1 };
    const index = draft.sections.findIndex((section) => section.id === widget.section);
    const section = draft.sections[Math.min(draft.sections.length - 1, Math.max(0, index + stepped.sections))]?.id ?? widget.section;
    const target = stepped.sections === 0 ? { section, column: here.column + stepped.columns, row: Math.max(1, here.row + stepped.rows) } : { section, column: here.column, row: 1 };
    moveTo(uid, target);
  };

  const place = (section: string, widget: LibraryWidget) => {
    history.change(t("steps.added"), (current) => placeWidget(current, widget.id, section, { width: widget.width, height: widget.height })[0]);
    setPicking(null);
  };

  const submit = (at: string | null) => {
    setConflict(false);
    setErrors(null);
    const sent = draft;
    save.mutate(
      { layout: toRequest(sent), revision: at },
      {
        onSuccess: (stored) => {
          dropDraft();
          setRevision(stored.revision);
          history.reset(sent);
          toast.success(t("saved"));
        },
        onError: (error) => {
          if (error instanceof ConflictError) {
            setConflict(true);
          } else if (error instanceof ValidationError) {
            setErrors(placeErrors(error.fields, sent, sent));
          } else {
            toast.error(error.message);
          }
        },
      },
    );
  };

  const overwrite = async () => {
    const fresh = await fetchLayout();
    setRevision(fresh.revision);
    submit(fresh.revision);
  };

  return (
    <div className="grid gap-6">
      <div className="glass-panel sticky top-3 z-20 flex flex-wrap items-center gap-3 rounded-xl px-4 py-3">
        <p className="text-sm text-muted-foreground" role="status">
          {dirty ? t("unsaved") : null}
        </p>
        <Button asChild variant="ghost" size="sm">
          <Link href={routes.adminLibrary}>
            <Library aria-hidden />
            {t("library")}
          </Link>
        </Button>
        <div className="ml-auto flex gap-2">
          <Button variant="ghost" size="icon" aria-label={t("undo")} title={t("undo")} disabled={!history.canUndo} onClick={history.undo}>
            <Undo2 aria-hidden />
          </Button>
          <Button variant="ghost" size="icon" aria-label={t("redo")} title={t("redo")} disabled={!history.canRedo} onClick={history.redo}>
            <Redo2 aria-hidden />
          </Button>
          <Button
            variant="outline"
            disabled={!dirty || save.isPending}
            onClick={() => {
              dropDraft();
              history.reset(base);
            }}
          >
            {t("discard")}
          </Button>
          <Button disabled={!dirty || save.isPending} onClick={() => submit(revision)}>
            {save.isPending ? t("saving") : t("save")}
          </Button>
        </div>
      </div>
      {conflict ? <ConflictNotice pending={save.isPending} onReload={onReload} onOverwrite={() => void overwrite()} /> : null}
      {errors ? <ErrorNotice title={t("invalid")} description={errors.other.join("\n") || undefined} /> : null}
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={dropped}
        accessibility={{ announcements, screenReaderInstructions: { draggable: t("announce.instructions") } }}
      >
        <SortableContext items={draft.sections.map((section) => `${SECTION_SORT_PREFIX}${section.id}`)} strategy={verticalListSortingStrategy}>
          <div className="grid gap-4">
            {draft.sections.map((section) => {
              const widgets = widgetsOf(draft, section.id);
              const moving = ghost && ghost.section === section.id ? draft.widgets.find((widget) => widget.uid === ghost.uid) : null;
              return (
                <SectionBlock
                  key={section.id}
                  section={section}
                  count={draft.sections.length}
                  widgetIds={widgets.map((widget) => widget.uid)}
                  ghost={ghost && moving ? { column: ghost.column, row: ghost.row, width: moving.width, rows: moving.height === "auto" ? 1 : moving.height } : null}
                  errors={errors?.sections[section.id] ?? []}
                  onRename={(title) => history.change(t("steps.renamed"), (current) => renameSection(current, section.id, title), true)}
                  onRemove={() => history.change(t("steps.sectionRemoved"), (current) => removeSection(current, section.id))}
                  onAdd={() => setPicking(section.id)}
                  onSettings={() => setSectionEditing(section.id)}
                >
                  {widgets.map((widget) => {
                    const entry = entryOf(widget);
                    return (
                      <WidgetTile
                        key={widget.uid}
                        widget={widget}
                        title={titleOf(widget.uid)}
                        type={entry?.type ?? widget.widget}
                        known={entry !== null && kindOf(entry.type) !== null}
                        errors={errors?.widgets[widget.uid] ?? []}
                        content={entry ? renderWidget(entry, widget) : null}
                        onResize={(size) => resize(widget.uid, size)}
                        onMove={(target) => moveTo(widget.uid, target)}
                        onStep={(stepped) => step(widget.uid, stepped)}
                        onPreviewMove={(target) => setGhost(target ? { ...target, uid: widget.uid } : null)}
                        onConfigure={() => {
                          if (entry && onBuild && builds?.(entry.type)) {
                            keepDraft(base, draft);
                            onBuild(entry.id, widget.uid);
                          } else {
                            setConfiguring(widget.widget);
                          }
                        }}
                        onRemove={() => history.change(t("steps.removed", { title: titleOf(widget.uid) }), (current) => removeWidget(current, widget.uid))}
                      />
                    );
                  })}
                </SectionBlock>
              );
            })}
          </div>
        </SortableContext>
      </DndContext>
      <div aria-live="polite" className="sr-only" data-resize-notice="">
        {notice}
      </div>
      <Button variant="outline" className="justify-self-start" onClick={() => history.change(t("steps.sectionAdded"), (current) => addSection(current, null))}>
        <Plus aria-hidden />
        {t("addSection")}
      </Button>
      {configuring ? renderSettings(configuring, () => setConfiguring(null), (next) => setRevision(next)) : null}
      <LibraryPicker open={picking !== null} kinds={kinds} onChoose={(widget) => picking && place(picking, widget)} onClose={() => setPicking(null)} />
      <SectionDialog
        section={draft.sections.find((section) => section.id === sectionEditing) ?? null}
        onApply={(patch) => sectionEditing && history.change(t("steps.sectionChanged"), (current) => updateSection(current, sectionEditing, patch))}
        onClose={() => setSectionEditing(null)}
      />
    </div>
  );
}

