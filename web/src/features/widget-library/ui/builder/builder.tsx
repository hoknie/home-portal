"use client";

import { useTranslations } from "next-intl";
import { type KeyboardEvent, type PointerEvent, type ReactNode, useRef, useState } from "react";
import { toast } from "sonner";

import { type LibraryEntry, fetchLibrary, useSaveLibraryWidget } from "@/entities/dashboard";
import type { CustomWidgetData } from "@/entities/widget";
import { ConflictError, ValidationError } from "@/shared/api";
import { useLeaveGuard } from "@/shared/lib/leave-guard";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { ConflictNotice } from "@/shared/ui/conflict-notice";
import { Splitter, useStoredSize } from "@/shared/ui/splitter";

import { textOf } from "../../model/block-drop";
import { blockAt, fieldOfPath, missingIn, pathOfField, slotAt, slotPaths, unmarked, withoutSlots } from "../../model/block-tree";
import { type BlockKind, errorsUnder, newBlock, tabOfError } from "../../model/blocks";
import type { WidgetKind } from "../../model/catalog";
import { useFilterSuggestions } from "../../model/filter-suggestions";
import { itemPaths, knownPaths, suggestionsOf } from "../../model/suggestion-source";
import { useBlockDrag } from "../../model/use-block-drag";
import { useBuilder } from "../../model/use-builder";
import { useWidgetPreview } from "../../model/use-widget-preview";
import { AccessTab } from "../settings-dialog/access-tab";
import { DataTab } from "../settings-dialog/data-tab";
import { LookTab } from "../settings-dialog/look-tab";
import { SuggestingContext } from "../settings-dialog/template-field";
import { Canvas, type CanvasSize, SizeControls } from "./canvas";
import { CollapseButton, SideColumn } from "./side-column";
import { type PanelTab, SettingsPanel } from "./settings-panel";
import { Toolbar } from "./toolbar";
import { DataTree } from "./data-tree";
import { DropMarker } from "./drop-marker";
import { type InspectorAction, Inspector } from "./inspector";
import { Outline } from "./outline";
import { Palette } from "./palette";
import { Pane } from "./pane";

export type BuilderMarking = { selected: string | null; flagged: readonly string[]; slots: readonly string[] };

export type WidgetBuilderProps = {
  entry: LibraryEntry;
  revision: string | null;
  isNew: boolean;
  kind: WidgetKind | null;
  environments: string[];
  initialSize: CanvasSize;
  renderCanvas: (entry: LibraryEntry, data: CustomWidgetData | null, marking: BuilderMarking) => ReactNode;
  onSaved: (id: string, revision: string | null, size: CanvasSize) => Promise<string | null>;
  onReload: () => void;
  onBack: () => void;
};

const SIZES = { left: 288, right: 384, tree: 320 } as const;

function insertAt(text: string, caret: number | null, token: string) {
  const at = caret ?? text.length;
  return text.slice(0, at) + token + text.slice(at);
}

export function WidgetBuilder({ entry, revision, isNew, kind, environments, initialSize, renderCanvas, onSaved, onReload, onBack }: WidgetBuilderProps) {
  const t = useTranslations();
  const builder = useBuilder(entry);
  const { draft, blocks, selected } = builder;
  const [tab, setTab] = useState<PanelTab>(isNew ? "data" : "block");
  const [size, setSize] = useState(initialSize);
  const [savedSize, setSavedSize] = useState(initialSize);
  const [leftWidth, setLeftWidth] = useStoredSize("home-portal.builder.left", SIZES.left);
  const [rightWidth, setRightWidth] = useStoredSize("home-portal.builder.right", SIZES.right);
  const [treeHeight, setTreeHeight] = useStoredSize("home-portal.builder.tree", SIZES.tree);
  const [leftShut, setLeftShut] = useStoredSize("home-portal.builder.left-shut", 2);
  const [rightShut, setRightShut] = useStoredSize("home-portal.builder.right-shut", 2);
  const resized = size.width !== savedSize.width || size.height !== savedSize.height;
  const [saveErrors, setSaveErrors] = useState<{ field: string; message: string }[]>([]);
  const [conflict, setConflict] = useState(false);
  const [leaving, setLeaving] = useState(false);
  const [held, setHeld] = useState(revision);
  const caret = useRef<{ field: string; caret: number | null } | null>(null);
  const save = useSaveLibraryWidget();
  const previewed = { ...draft.settings, blocks: unmarked(blocks) };
  const preview = useWidgetPreview(isNew ? null : entry.id, previewed, true);
  const filters = useFilterSuggestions();
  useLeaveGuard(builder.changed, t("widgetBuilder.leave"));
  const data = preview.preview?.data ?? null;
  const known = knownPaths(preview.preview?.paths ?? [], data);
  const errors = [...(preview.preview?.errors ?? []), ...saveErrors];
  const missing = missingIn(blocks);
  const flagged = [...new Set([...missing.map((found) => textOf(found.path)), ...errors.map((error) => pathOfField(error.field)?.[0]).filter((path) => path !== undefined).map(textOf)])];
  const kindLabel = (name: string) => t(`widgetBuilder.tree.kinds.${name}` as "widgetBuilder.tree.kinds.value");
  const groups = { data: t("layoutEditor.blocks.suggestions.data"), widget: t("layoutEditor.blocks.suggestions.widget"), item: t("layoutEditor.blocks.suggestions.item") };
  const suggesting = {
    names: (items: string | null) => suggestionsOf(known, itemPaths(items, data, known), groups, kindLabel),
    filters,
    caret: (field: string, at: number | null) => {
      caret.current = { field, caret: at };
    },
  };
  const block = selected === null ? undefined : blockAt(blocks, selected);
  const kindName = (target: { kind: string } | undefined) => (target ? t(`layoutEditor.blocks.kinds.${target.kind as BlockKind}`) : "");
  const drag = useBlockDrag(
    blocks,
    (source, drop) => builder.drop(source, drop, source.kind === "new" ? kindName(source.block) : kindName(blockAt(blocks, source.path))),
    (refused) => builder.announce({ key: "refused", params: { reason: refused } }),
  );

  const act = (action: InspectorAction) => {
    const label = kindName(block);
    if (action === "duplicate") {
      builder.duplicate(label);
    } else if (action === "delete") {
      builder.remove(label);
    } else {
      builder.step(action, label);
    }
  };

  const keys = (event: KeyboardEvent) => {
    const commands: Record<string, InspectorAction> = { ArrowUp: "up", ArrowDown: "down", ArrowLeft: "out", ArrowRight: "into" };
    if (event.altKey && commands[event.key]) {
      event.preventDefault();
      act(commands[event.key]);
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "d") {
      event.preventDefault();
      act("duplicate");
    } else if (event.key === "Delete") {
      event.preventDefault();
      act("delete");
    }
  };

  const insert = (path: string) => {
    if (selected === null || block === undefined) {
      return;
    }
    const target = caret.current;
    const field = target?.field ?? (typeof block.value === "string" ? "value" : "text");
    const current = typeof block[field] === "string" ? (block[field] as string) : "";
    builder.updateBlock(selected, { ...block, [field]: insertAt(current, target?.field === field ? target.caret : null, `{{${path}}}`) }, kindName(block));
  };

  const submit = async (at: string | null) => {
    setConflict(false);
    setSaveErrors([]);
    if (missing.length > 0) {
      builder.select(missing[0].path);
      setTab("block");
      toast.error(t("widgetBuilder.missing"));
      return;
    }
    try {
      const saved = await save.mutateAsync({
        entry: { ...draft, width: size.width, height: size.height, settings: { ...draft.settings, blocks: withoutSlots(blocks) } },
        existing: isNew ? null : entry.id,
        revision: at,
      });
      builder.markSaved(draft);
      setHeld(saved.revision);
      const after = await onSaved(saved.data.id, saved.revision, size);
      setHeld(after ?? saved.revision);
      setSavedSize(size);
      toast.success(t("widgetBuilder.saved"));
    } catch (error) {
      if (error instanceof ConflictError) {
        setConflict(true);
      } else if (error instanceof ValidationError) {
        setSaveErrors(error.fields);
        const first = error.fields.map((field) => pathOfField(field.field)).find((found) => found !== null);
        if (first) {
          builder.select(first[0]);
        }
        setTab(tabOfError(error.fields[0]?.field ?? "") === "data" ? "data" : "block");
      } else {
        toast.error(error instanceof Error ? error.message : String(error));
      }
    }
  };

  const failing = new Set<PanelTab>([
    ...errors.map((error) => (tabOfError(error.field) === "data" ? ("data" as const) : ("block" as const))),
    ...(missing.length > 0 ? (["block"] as const) : []),
  ]);
  const custom = preview.preview ? { blocks: preview.preview.blocks } : null;
  const ownErrors = selected === null ? {} : errorsUnder(errors, fieldOfPath(selected));
  const parentKind = selected === null || selected.length < 2 ? null : (blockAt(blocks, selected.slice(0, -1))?.kind ?? null);
  const startMove = (path: number[], event: PointerEvent) => drag.start({ kind: "move", path }, blockAt(blocks, path) ?? newBlock("text"), event);
  const treeTitle = t("widgetBuilder.outline.title");
  const panelTitle = t("widgetBuilder.panel.title");
  return (
    <div className={drag.dragging ? "flex h-[calc(100dvh-7rem)] min-h-[36rem] cursor-grabbing flex-col gap-3 select-none" : "flex h-[calc(100dvh-7rem)] min-h-[36rem] flex-col gap-3"} data-widget-builder="">
      <Toolbar
        title={draft.title ?? ""}
        placeholder={kind?.title ?? t("widgetBuilder.titlePlaceholder")}
        changed={builder.changed || resized}
        saving={save.isPending}
        canSave={!save.isPending && (builder.changed || resized || isNew)}
        canUndo={builder.history.canUndo}
        canRedo={builder.history.canRedo}
        onTitle={(title) => builder.updateEntry("title", (current) => ({ ...current, title: title === "" ? null : title }), true)}
        onBack={() => (builder.changed || resized ? setLeaving(true) : onBack())}
        onUndo={builder.history.undo}
        onRedo={builder.history.redo}
        onSave={() => void submit(held)}
      />
      {conflict ? (
        <ConflictNotice
          pending={save.isPending}
          onReload={onReload}
          onOverwrite={() =>
            void fetchLibrary().then((fresh) => {
              setHeld(fresh.revision);
              return submit(fresh.revision);
            })
          }
        />
      ) : null}
      <div aria-live="polite" className="sr-only" data-builder-notice="">
        {builder.announcement
          ? builder.announcement.key === "refused"
            ? t(`widgetBuilder.refused.${builder.announcement.params.reason as "tooDeep"}`)
            : t(`widgetBuilder.announce.${builder.announcement.key}`, builder.announcement.params)
          : null}
      </div>
      <SuggestingContext.Provider value={suggesting}>
        <div className="flex min-h-0 flex-1 gap-3 max-lg:flex-col" onKeyDown={keys}>
          <SideColumn side="left" title={treeTitle} collapsed={leftShut === 1} width={leftWidth} min={220} max={480} onToggle={() => setLeftShut(leftShut === 1 ? 2 : 1, true)} onWidth={setLeftWidth}>
            <Pane id="builder-outline" title={treeTitle} className="shrink-0" bodyClassName="p-3" actions={<CollapseButton side="left" title={treeTitle} onToggle={() => setLeftShut(1, true)} />} style={{ height: treeHeight }}>
              <Outline blocks={blocks} selected={selected} flagged={flagged} onSelect={builder.select} onKey={keys} onDragStart={startMove} />
            </Pane>
            <Splitter label={t("widgetBuilder.columns.resize", { title: treeTitle })} orientation="horizontal" value={treeHeight} min={120} max={900} onChange={setTreeHeight} className="-my-1.5 self-center" />
            <Pane id="builder-palette" title={t("widgetBuilder.palette.title")} className="min-h-32 flex-1" bodyClassName="p-3">
              <Palette
                onAdd={(name) => {
                  if (!drag.justDragged()) {
                    builder.add(name, kindName({ kind: name }));
                  }
                }}
                onDragStart={(name, event) => drag.start({ kind: "new", block: newBlock(name) }, newBlock(name), event)}
              />
            </Pane>
          </SideColumn>
          <Pane id="builder-canvas" title={t("widgetBuilder.canvas.title")} className="min-h-80 min-w-0 flex-1" actions={<SizeControls size={size} onSize={setSize} />}>
            <Canvas size={size} empty={blocks.length === 0} dragging={drag.dragging} onSelect={builder.select} onDragStart={startMove}>
              {renderCanvas(draft, custom, { selected: selected === null ? null : textOf(selected), flagged, slots: slotPaths(blocks) })}
            </Canvas>
          </Pane>
          <SideColumn side="right" title={panelTitle} collapsed={rightShut === 1} width={rightWidth} min={300} max={640} onToggle={() => setRightShut(rightShut === 1 ? 2 : 1, true)} onWidth={setRightWidth}>
            <SettingsPanel tab={tab} failing={failing} onTab={setTab} collapse={<CollapseButton side="right" title={panelTitle} onToggle={() => setRightShut(1, true)} />}>
              {tab === "block" ? (
                <Inspector
                  slot={slotAt(blocks, selected)}
                  block={block}
                  path={selected}
                  parentKind={parentKind}
                  errors={ownErrors}
                  missing={missing.filter((found) => selected !== null && textOf(found.path) === textOf(selected)).map((found) => found.field)}
                  onChange={(next) => selected !== null && builder.updateBlock(selected, next, kindName(next))}
                  onAction={act}
                >
                  <DataTree known={known} running={preview.running} canRun={draft.settings.source !== undefined} onRun={preview.run} onInsert={insert} />
                </Inspector>
              ) : null}
              {tab === "data" ? (
                <DataTab
                  settings={draft.settings}
                  errors={Object.fromEntries(errors.filter((error) => tabOfError(error.field) === "data").map((error) => [error.field, error.message]))}
                  preview={preview}
                  onChange={(settings) => builder.updateEntry("data", (current) => ({ ...current, settings }), true)}
                />
              ) : null}
              {tab === "look" ? <LookTab value={draft.appearance} onChange={(appearance) => builder.updateEntry("look", (current) => ({ ...current, appearance }))} /> : null}
              {tab === "access" ? (
                <AccessTab value={{ environments: draft.environments, public: draft.public }} environments={environments} custom onChange={(access) => builder.updateEntry("access", (current) => ({ ...current, ...access }))} />
              ) : null}
            </SettingsPanel>
          </SideColumn>
        </div>
      </SuggestingContext.Provider>
      <DropMarker drop={drag.drop} ghost={drag.ghost} />
      <ConfirmDialog
        open={leaving}
        onOpenChange={setLeaving}
        title={t("layoutEditor.dialog.discardTitle")}
        description={t("layoutEditor.dialog.discardDescription")}
        confirmLabel={t("layoutEditor.dialog.discard")}
        onConfirm={() => {
          setLeaving(false);
          onBack();
        }}
      />
    </div>
  );
}
