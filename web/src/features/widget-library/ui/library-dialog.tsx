"use client";

import type { ReactNode } from "react";

import { type LibraryEntry, useLibrary, useSaveLibraryWidget } from "@/entities/dashboard";
import type { CustomWidgetData } from "@/entities/widget";

import type { WidgetKind } from "../model/catalog";
import { type DialogTab, SettingsDialog } from "./settings-dialog/settings-dialog";

export type LibraryDialogProps = {
  entry: LibraryEntry | null;
  kinds: WidgetKind[];
  environments: string[];
  firstTab?: DialogTab;
  renderPreview: (entry: LibraryEntry, custom: CustomWidgetData | null) => ReactNode;
  onSaved?: (revision: string | null, id: string) => void;
  onClose: () => void;
};

export function LibraryDialog({ entry, kinds, environments, firstTab, renderPreview, onSaved, onClose }: LibraryDialogProps) {
  const library = useLibrary();
  const save = useSaveLibraryWidget();
  const known = new Set((library.data?.data.widgets ?? []).map((widget) => widget.id));
  return (
    <SettingsDialog
      entry={entry}
      kind={entry ? (kinds.find((kind) => kind.type === entry.type) ?? null) : null}
      environments={environments}
      firstTab={firstTab}
      renderPreview={renderPreview}
      onClose={onClose}
      onSave={async (edited) => {
        const existing = edited.id !== null && known.has(edited.id) ? edited.id : null;
        const saved = await save.mutateAsync({ entry: edited, existing, revision: library.data?.revision ?? null });
        onSaved?.(saved.revision, saved.data.id);
      }}
    />
  );
}
