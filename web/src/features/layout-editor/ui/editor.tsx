"use client";

import {
  DndContext,
  type DragEndEvent,
  KeyboardSensor,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { SortableContext, sortableKeyboardCoordinates, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { Plus } from "lucide-react";
import { useTranslations } from "next-intl";
import { type ReactNode, useMemo, useState } from "react";
import { toast } from "sonner";

import { type Dashboard, fetchLayout, useSaveLayout } from "@/entities/dashboard";
import { ConflictError, type Revisioned, ValidationError, type WidgetSize } from "@/shared/api";
import { useLeaveGuard } from "@/shared/lib/leave-guard";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { Button } from "@/shared/ui/primitives";

import type { PreviewScope, WidgetKind } from "../model/catalog";
import { SECTION_SORT_PREFIX, afterDrop, byKind, keyboardByKind } from "../model/drag";
import {
  type Draft,
  addSection,
  addWidget,
  fromLayout,
  removeSection,
  removeWidget,
  renameSection,
  sameDraft,
  toRequest,
  updateWidget,
  widgetsOf,
} from "../model/draft";
import { type PlacedErrors, placeErrors } from "../model/errors";
import { useAnnouncements } from "../model/use-announcements";
import { PreviewPanel } from "./preview-panel";
import { SectionBlock } from "./section-block";
import { WidgetSheet } from "./widget-sheet";
import { WidgetTile } from "./widget-tile";

export type EditorProps = {
  loaded: Revisioned<Dashboard>;
  kinds: WidgetKind[];
  environments: string[];
  spanOf: (size: WidgetSize) => string;
  renderPreview: (draft: Draft, scope: PreviewScope) => ReactNode;
  onReload: () => void;
};

export function Editor({ loaded, kinds, environments, spanOf, renderPreview, onReload }: EditorProps) {
  const t = useTranslations("layoutEditor");
  const base = useMemo(() => fromLayout(loaded.data), [loaded]);
  const [draft, setDraft] = useState<Draft>(base);
  const [revision, setRevision] = useState(loaded.revision);
  const [editing, setEditing] = useState<string | null>(null);
  const [conflict, setConflict] = useState(false);
  const [errors, setErrors] = useState<PlacedErrors | null>(null);
  const [notice, setNotice] = useState("");
  const save = useSaveLayout();
  const dirty = !sameDraft(draft, base);
  useLeaveGuard(dirty, t("leave"));
  const kindOf = (type: string) => kinds.find((kind) => kind.type === type) ?? null;
  const titleOf = (uid: string) => {
    const widget = draft.widgets.find((candidate) => candidate.uid === uid);
    return widget ? (widget.title ?? kindOf(widget.type)?.title ?? widget.type) : uid;
  };
  const announcements = useAnnouncements(draft, titleOf);
  const sensors = useSensors(useSensor(PointerSensor), useSensor(KeyboardSensor, { coordinateGetter: keyboardByKind(sortableKeyboardCoordinates) }));

  const dropped = ({ active, over }: DragEndEvent) => {
    const item = (entry: { id: string | number; data: { current?: Record<string, unknown> } }) => ({
      id: String(entry.id),
      kind: (entry.data.current?.kind as string | undefined) ?? null,
      section: (entry.data.current?.section as string | undefined) ?? null,
    });
    setDraft((current) => afterDrop(current, item(active), over ? item(over) : null));
  };

  const resize = (uid: string, size: WidgetSize) => {
    setDraft((current) => updateWidget(current, uid, { size }));
    setNotice(t("announce.resized", { title: titleOf(uid), size: t(`sizes.${size}`) }));
  };

  const submit = (at: string | null) => {
    setConflict(false);
    setErrors(null);
    const sent = draft;
    save.mutate(
      { layout: toRequest(sent), revision: at },
      {
        onSuccess: () => toast.success(t("saved")),
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

  const editingWidget = draft.widgets.find((widget) => widget.uid === editing) ?? null;
  return (
    <div className="grid gap-6">
      <div className="glass-panel sticky top-3 z-20 flex flex-wrap items-center gap-3 rounded-xl px-4 py-3">
        <p className="text-sm text-muted-foreground" role="status">
          {dirty ? t("unsaved") : null}
        </p>
        <div className="ml-auto flex gap-2">
          <Button variant="outline" disabled={!dirty || save.isPending} onClick={() => setDraft(base)}>
            {t("discard")}
          </Button>
          <Button disabled={!dirty || save.isPending} onClick={() => submit(revision)}>
            {save.isPending ? t("saving") : t("save")}
          </Button>
        </div>
      </div>
      {conflict ? (
        <ErrorNotice
          title={t("conflict")}
          action={
            <div className="flex flex-wrap gap-2">
              <Button variant="outline" size="sm" onClick={onReload}>
                {t("reload")}
              </Button>
              <Button size="sm" onClick={() => void overwrite()}>
                {t("overwrite")}
              </Button>
            </div>
          }
        />
      ) : null}
      {errors ? <ErrorNotice title={t("invalid")} description={errors.other.join("\n") || undefined} /> : null}
      <DndContext
        sensors={sensors}
        collisionDetection={byKind(closestCenter)}
        onDragEnd={dropped}
        accessibility={{ announcements, screenReaderInstructions: { draggable: t("announce.instructions") } }}
      >
        <SortableContext items={draft.sections.map((section) => `${SECTION_SORT_PREFIX}${section.id}`)} strategy={verticalListSortingStrategy}>
        <div className="grid gap-4">
          {draft.sections.map((section) => {
            const widgets = widgetsOf(draft, section.id);
            return (
              <SectionBlock
                key={section.id}
                section={section}
                count={draft.sections.length}
                widgetIds={widgets.map((widget) => widget.uid)}
                kinds={kinds}
                errors={errors?.sections[section.id] ?? []}
                onRename={(title) => setDraft((current) => renameSection(current, section.id, title))}
                onRemove={() => setDraft((current) => removeSection(current, section.id))}
                onAdd={(type) => setDraft((current) => addWidget(current, type, section.id))}
              >
                {widgets.map((widget) => (
                  <WidgetTile
                    key={widget.uid}
                    widget={widget}
                    title={titleOf(widget.uid)}
                    known={kindOf(widget.type) !== null}
                    spanOf={spanOf}
                    errors={errors?.widgets[widget.uid] ?? []}
                    onResize={(size) => resize(widget.uid, size)}
                    onEdit={() => setEditing(widget.uid)}
                    onRemove={() => setDraft((current) => removeWidget(current, widget.uid))}
                  />
                ))}
              </SectionBlock>
            );
          })}
        </div>
        </SortableContext>
      </DndContext>
      <div aria-live="polite" className="sr-only" data-resize-notice="">
        {notice}
      </div>
      <Button variant="outline" className="justify-self-start" onClick={() => setDraft((current) => addSection(current, null))}>
        <Plus aria-hidden />
        {t("addSection")}
      </Button>
      <PreviewPanel environments={environments} render={(scope) => renderPreview(draft, scope)} />
      <WidgetSheet
        widget={editingWidget}
        kind={editingWidget ? kindOf(editingWidget.type) : null}
        title={editing ? titleOf(editing) : ""}
        environments={environments}
        onChange={(patch) => editing && setDraft((current) => updateWidget(current, editing, patch))}
        onClose={() => setEditing(null)}
      />
    </div>
  );
}
