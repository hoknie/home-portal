"use client";

import { useTranslations } from "next-intl";
import { useEffect } from "react";

import { routes } from "@/shared/config";

export const ID_PARAMETER = "id";

export function legacyTarget(search: string) {
  const id = new URLSearchParams(search).get(ID_PARAMETER);
  return id ? routes.workflowEdit(id) : routes.adminWorkflows;
}

export function LegacyWorkflowEdit() {
  const t = useTranslations("common");
  useEffect(() => {
    window.location.replace(legacyTarget(window.location.search));
  }, []);
  return (
    <p className="p-6 text-sm text-muted-foreground" aria-busy="true">
      {t("redirecting")}
    </p>
  );
}
