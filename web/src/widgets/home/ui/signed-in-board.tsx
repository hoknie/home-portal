"use client";

import { LayoutDashboard } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";

import { useDashboard } from "@/entities/dashboard";
import { useServices } from "@/entities/service";
import { BoardGrid } from "@/features/widget-board";
import { routes } from "@/shared/config";
import { EmptyState } from "@/shared/ui/empty-state";
import { Button } from "@/shared/ui/primitives";

import { BoardState } from "./board-state";

export function SignedInBoard() {
  const t = useTranslations("home");
  const dashboard = useDashboard();
  const services = useServices();
  const ready = dashboard.data && services.data;
  return (
    <>
      {ready ? (
        dashboard.data.widgets.length > 0 ? (
          <BoardGrid sections={dashboard.data.sections} widgets={dashboard.data.widgets} services={services.data.data.services} />
        ) : (
          <EmptyState
            icon={LayoutDashboard}
            title={t("empty")}
            description={t("emptyHint")}
            action={
              <Button asChild>
                <Link href={routes.adminLayout}>{t("editLayout")}</Link>
              </Button>
            }
          />
        )
      ) : (
        <BoardState
          error={dashboard.error ?? services.error}
          onRetry={() => {
            void dashboard.refetch();
            void services.refetch();
          }}
        />
      )}
    </>
  );
}
