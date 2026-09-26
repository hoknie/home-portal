"use client";

import { useMutation } from "@tanstack/react-query";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import type { Network } from "@/entities/network";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";

import { restartPortal } from "../api/restart";
import { restartTarget } from "../model/restart-target";
import { RestartingScreen } from "./restarting-screen";

export function useRestartFlow(network?: Network | null) {
  const t = useTranslations("restart");
  const [open, setOpen] = useState(false);
  const [target, setTarget] = useState<string | null>(null);
  const restart = useMutation({ mutationFn: restartPortal });
  const confirm = async () => {
    try {
      await restart.mutateAsync();
      setTarget(restartTarget(window.location, network));
    } catch {
      toast.error(t("failed"));
    }
    setOpen(false);
  };
  const element = (
    <>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={t("title")}
        description={t("description")}
        confirmLabel={t("action")}
        pending={restart.isPending}
        onConfirm={confirm}
      />
      {target ? <RestartingScreen target={target} /> : null}
    </>
  );
  return { ask: () => setOpen(true), element };
}
