"use client";

import { RotateCcw } from "lucide-react";
import { useTranslations } from "next-intl";

import { DropdownMenuItem } from "@/shared/ui/primitives";

import { useRestartFlow } from "./use-restart-flow";

export function RestartPortalItem() {
  const t = useTranslations("restart");
  const flow = useRestartFlow();
  return (
    <>
      <DropdownMenuItem
        onSelect={(event) => {
          event.preventDefault();
          flow.ask();
        }}
      >
        <RotateCcw aria-hidden />
        {t("action")}
      </DropdownMenuItem>
      {flow.element}
    </>
  );
}
