"use client";

import { House } from "lucide-react";
import { useTranslations } from "next-intl";

import { usePortal } from "@/entities/portal";
import { BoardGrid } from "@/features/widget-board";
import { EmptyState } from "@/shared/ui/empty-state";

import { publicGrid } from "../model/board";
import { BoardState } from "./board-state";

export function PublicBoard() {
  const t = useTranslations("publicPortal");
  const portal = usePortal();
  const data = portal.data;
  const widgets = data ? publicGrid(data) : [];
  const listed = widgets.some((widget) => widget.type === "services");
  const extra = data && !listed && data.services.length > 0;
  const sections = data ? (data.sections.length > 0 ? data.sections : [{ id: "main", title: null }]) : [];
  return (
    <>
      {data ? (
        widgets.length > 0 || data.services.length > 0 ? (
          <BoardGrid
            sections={extra ? [...sections, { id: "public-services", title: null }] : sections}
            widgets={
              extra
                ? [
                    ...widgets,
                    { key: "public-services", type: "services", id: null, title: null, settings: {}, section: "public-services", size: "full" },
                  ]
                : widgets
            }
            services={data.services}
            scope="public"
          />
        ) : (
          <EmptyState icon={House} title={t("empty")} description={t("emptyHint")} />
        )
      ) : (
        <BoardState error={portal.error} onRetry={() => void portal.refetch()} />
      )}
    </>
  );
}
