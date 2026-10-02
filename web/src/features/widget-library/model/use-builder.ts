"use client";

import { useState } from "react";

import type { LibraryEntry } from "@/entities/dashboard";
import { useDraftHistory } from "@/shared/lib/history";

import { type Drop, type DragSource, dropped } from "./block-drop";
import { type BlockKind, type RawBlock, blocksOf, newBlock } from "./blocks";
import { type BlockPath, type Command, type Refusal, blockAt, duplicateAt, insertAt, isGroup, removeAt, samePath, slotAt, stepped, updateAt } from "./block-tree";

export type Announcement = { key: "added" | "moved" | "deleted" | "duplicated" | "undone" | "redone" | "refused"; params: Record<string, string | number> };

export type Builder = ReturnType<typeof useBuilder>;

function sameEntry(left: LibraryEntry, right: LibraryEntry) {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function placeOf(path: BlockPath) {
  return path.map((index) => index + 1).join(".");
}

export function useBuilder(entry: LibraryEntry) {
  const [announcement, setAnnouncement] = useState<Announcement | null>(null);
  const history = useDraftHistory(entry, sameEntry, (kind, label) => setAnnouncement({ key: kind, params: { change: label } }));
  const [selected, setSelected] = useState<BlockPath | null>(null);
  const [base, setBase] = useState(entry);
  const draft = history.draft;
  const blocks = blocksOf(draft.settings);

  const withBlocks = (current: LibraryEntry, next: RawBlock[]): LibraryEntry => ({ ...current, settings: { ...current.settings, blocks: next } });

  const changeBlocks = (label: string, update: (current: RawBlock[]) => [RawBlock[], BlockPath | null], coalesce = false) => {
    const [next, focus] = update(blocks);
    history.change(label, (current) => withBlocks(current, next), coalesce);
    setSelected(focus);
    return focus;
  };

  const add = (kind: BlockKind, label: string) => {
    const block = newBlock(kind);
    if (slotAt(blocks, selected)) {
      const slot = selected as BlockPath;
      changeBlocks(label, (current) => [updateAt(current, slot, block), slot]);
      setAnnouncement({ key: "added", params: { kind: label, place: placeOf(slot) } });
      return;
    }
    const target = selected === null ? { parent: [], index: blocks.length } : isGroup(blockAt(blocks, selected)) ? { parent: selected, index: 0 } : { parent: selected.slice(0, -1), index: selected[selected.length - 1] + 1 };
    const path = changeBlocks(label, (current) => [insertAt(current, target, block), [...target.parent, target.index]]);
    setAnnouncement({ key: "added", params: { kind: label, place: placeOf(path ?? []) } });
  };

  const drop = (source: DragSource, where: Drop, label: string) => {
    const path = changeBlocks(label, (current) => dropped(current, source, where));
    setAnnouncement({ key: source.kind === "new" ? "added" : "moved", params: { kind: label, place: placeOf(path ?? []) } });
  };

  const step = (command: Command, label: string): Refusal | null => {
    if (selected === null) {
      return null;
    }
    const result = stepped(blocks, selected, command);
    if (result === null || !Array.isArray(result)) {
      if (result !== null) {
        setAnnouncement({ key: "refused", params: { reason: result } });
      }
      return result;
    }
    changeBlocks(label, () => result);
    setAnnouncement({ key: "moved", params: { kind: label, place: placeOf(result[1]) } });
    return null;
  };

  const duplicate = (label: string) => {
    if (selected !== null) {
      const path = changeBlocks(label, (current) => duplicateAt(current, selected));
      setAnnouncement({ key: "duplicated", params: { kind: label, place: placeOf(path ?? []) } });
    }
  };

  const remove = (label: string) => {
    if (selected !== null) {
      const removed = selected;
      changeBlocks(label, (current) => [removeAt(current, removed), null]);
      setAnnouncement({ key: "deleted", params: { kind: label, place: placeOf(removed) } });
    }
  };

  const updateBlock = (path: BlockPath, block: RawBlock, label: string) =>
    history.change(label, (current) => withBlocks(current, updateAt(blocksOf(current.settings), path, block)), true);

  const updateEntry = (label: string, update: (current: LibraryEntry) => LibraryEntry, coalesce = false) => history.change(label, update, coalesce);

  return {
    draft,
    blocks,
    selected,
    select: (path: BlockPath | null) => setSelected(path),
    isSelected: (path: BlockPath) => selected !== null && samePath(selected, path),
    add,
    drop,
    step,
    duplicate,
    remove,
    updateBlock,
    updateEntry,
    announcement,
    announce: setAnnouncement,
    history,
    changed: !sameEntry(draft, base),
    markSaved: (saved: LibraryEntry) => setBase(saved),
  };
}
