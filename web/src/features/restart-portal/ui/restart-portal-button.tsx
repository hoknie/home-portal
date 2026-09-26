"use client";

import { RotateCcw } from "lucide-react";
import { useTranslations } from "next-intl";

import type { Network } from "@/entities/network";
import { Button } from "@/shared/ui/primitives";

import { useRestartFlow } from "./use-restart-flow";

export type RestartPortalButtonProps = { network: Network };

export function RestartPortalButton({ network }: RestartPortalButtonProps) {
  const t = useTranslations("restart");
  const flow = useRestartFlow(network);
  return (
    <>
      <Button type="button" size="sm" variant="outline" onClick={flow.ask}>
        <RotateCcw aria-hidden />
        {t("now")}
      </Button>
      {flow.element}
    </>
  );
}
