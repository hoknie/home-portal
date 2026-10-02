"use client";

import { AlertTriangle } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button } from "@/shared/ui/kit";

export type ConflictNoticeProps = {
  pending?: boolean;
  onReload: () => void;
  onOverwrite: () => void;
};

export function ConflictNotice({ pending = false, onReload, onOverwrite }: ConflictNoticeProps) {
  const t = useTranslations("conflict");
  const [confirming, setConfirming] = useState(false);
  return (
    <div role="alert" className="flex items-start gap-3 rounded-xl border border-destructive/30 bg-destructive/5 p-4">
      <AlertTriangle className="mt-0.5 size-5 shrink-0 text-destructive" aria-hidden />
      <div className="flex-1 space-y-1">
        <p className="text-sm font-medium">{t("title")}</p>
        <p className="text-sm text-muted-foreground">{t("description")}</p>
        <div className="flex flex-wrap gap-2 pt-2">
          <Button type="button" size="sm" variant="outline" disabled={pending} onClick={() => setConfirming(true)}>
            {t("reload")}
          </Button>
          <Button type="button" size="sm" variant="destructive" disabled={pending} onClick={onOverwrite}>
            {t("overwrite")}
          </Button>
        </div>
      </div>
      <ConfirmDialog
        open={confirming}
        title={t("reloadTitle")}
        description={t("reloadDescription")}
        confirmLabel={t("reload")}
        onOpenChange={setConfirming}
        onConfirm={() => {
          setConfirming(false);
          onReload();
        }}
      />
    </div>
  );
}
