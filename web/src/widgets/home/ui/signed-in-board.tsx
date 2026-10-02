"use client";

import { LayoutDashboard } from "lucide-react";
import { useTranslations } from "next-intl";

import { Allowed } from "@/entities/session";
import { useDashboard } from "@/entities/dashboard";
import { useServices } from "@/entities/service";
import { BoardGrid } from "@/features/widget-board";
import { routes } from "@/shared/config";
import { EmptyState, LinkButton, Loaded } from "@/shared/ui/kit";

import { BoardSkeleton } from "./board-state";

export function SignedInBoard() {
  const t = useTranslations("home");
  const dashboard = useDashboard();
  const services = useServices();
  return (
    <Loaded.all queries={[dashboard, services]} skeleton={<BoardSkeleton />}>
      {(layout, listed) =>
        layout.widgets.length > 0 ? (
          <BoardGrid sections={layout.sections} widgets={layout.widgets} services={listed.data.services} />
        ) : (
          <EmptyState
            icon={LayoutDashboard}
            title={t("empty")}
            description={t("emptyHint")}
            action={
              <Allowed area="layout" action="update">
                <LinkButton href={routes.adminLayout} variant="default">
                  {t("editLayout")}
                </LinkButton>
              </Allowed>
            }
          />
        )
      }
    </Loaded.all>
  );
}
