"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { type LibraryWidget, useLayout } from "@/entities/dashboard";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { Skeleton } from "@/shared/ui/primitives";

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
      <Skeleton className="h-64 w-full" aria-busy="true" />
    );
  }
  return <Editor key={`${layout.data.revision}-${layout.dataUpdatedAt}`} loaded={layout.data} onReload={() => void layout.refetch()} {...props} />;
}
