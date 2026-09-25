"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { useLayout } from "@/entities/dashboard";
import type { WidgetSize } from "@/shared/api";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { Skeleton } from "@/shared/ui/primitives";

import type { PreviewScope, WidgetKind } from "../model/catalog";
import type { Draft } from "../model/draft";
import { Editor } from "./editor";

export type LayoutEditorProps = {
  kinds: WidgetKind[];
  environments: string[];
  spanOf: (size: WidgetSize) => string;
  renderPreview: (draft: Draft, scope: PreviewScope) => ReactNode;
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
