"use client";

import { useTranslations } from "next-intl";

import { absoluteAddress, curlExample } from "@/entities/webhook";
import { CopyLine } from "@/shared/ui/copy-line";
import { Button, Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/shared/ui/primitives";

export type TokenDialogProps = { token: string | null; address: string; variables: string[]; onClose: () => void };

export function TokenDialog({ token, address, variables, onClose }: TokenDialogProps) {
  const t = useTranslations("webhooks");
  const url = typeof window === "undefined" ? address : absoluteAddress(address, window.location.origin);
  return (
    <Dialog open={token !== null} onOpenChange={(open) => (open ? undefined : onClose())}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t("tokenTitle")}</DialogTitle>
          <DialogDescription>{t("tokenOnce")}</DialogDescription>
        </DialogHeader>
        {token ? (
          <div className="grid gap-4">
            <CopyLine label={t("token")} text={token} />
            <CopyLine label={t("example")} text={curlExample(url, token, variables)} />
          </div>
        ) : null}
        <DialogFooter>
          <Button type="button" onClick={onClose}>
            {t("tokenSaved")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
