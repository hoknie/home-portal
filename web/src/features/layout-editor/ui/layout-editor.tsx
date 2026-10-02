"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { type LibraryWidget, useLayout } from "@/entities/dashboard";
import { Appear, ErrorNotice, SkeletonWidget } from "@/shared/ui/kit";

import type { KindLabel } from "../model/catalog";
import type { DraftWidget } from "../model/draft";
import { Editor } from "./editor";

export type LayoutEditorProps = {
  kinds: KindLabel[];
  renderWidget: (widget: LibraryWidget, placement: DraftWidget) => ReactNode;
  renderSettings: (id: string, close: () => void, adopt: (revision: string | null) => void) => ReactNode;
  builds?: (type: string) => boolean;
  onBuild?: (id: string, place: string) => void;
};

export function LayoutEditor(props: LayoutEditorProps) {
  const t = useTranslations("errors");
  const layout = useLayout();
  if (!layout.data) {
    return layout.error ? (
      <ErrorNotice title={t("loadFailed")} description={layout.error.message} onRetry={() => void layout.refetch()} />
    ) : (
      <div className="grid gap-4" data-skeleton="layout" aria-busy="true">
        <SkeletonWidget rows={1} />
        <SkeletonWidget rows={3} />
      </div>
    );
  }
  return (
    <Appear>
      <Editor key={`${layout.data.revision}-${layout.dataUpdatedAt}`} loaded={layout.data} onReload={() => void layout.refetch()} {...props} />
    </Appear>
  );
}
