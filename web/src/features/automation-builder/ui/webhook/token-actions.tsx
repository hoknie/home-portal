"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type Webhook, useIssueToken, useRemoveToken } from "@/entities/webhook";
import { ConflictError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Badge, Button } from "@/shared/ui/primitives";

import { TokenDialog } from "./token-dialog";

export function TokenActions({ webhook, revision }: { webhook: Webhook; revision: string | null }) {
  const t = useTranslations();
  const issue = useIssueToken();
  const remove = useRemoveToken();
  const [token, setToken] = useState<string | null>(null);
  const [removing, setRemoving] = useState(false);
  const failed = (error: unknown) => toast.error(t(error instanceof ConflictError ? "errors.conflict" : "errors.generic"));
  const create = async () => {
    try {
      setToken(await issue.mutateAsync({ id: webhook.id, revision }));
    } catch (error) {
      failed(error);
    }
  };
  const drop = async () => {
    try {
      await remove.mutateAsync({ id: webhook.id, revision });
      toast.success(t("webhooks.tokenRemoved"));
    } catch (error) {
      failed(error);
    }
    setRemoving(false);
  };
  return (
    <div className="flex flex-wrap items-center gap-2">
      <Badge variant={webhook.protected ? "outline" : "destructive"}>{t(webhook.protected ? "webhooks.protected" : "webhooks.open")}</Badge>
      <Button type="button" variant="outline" size="sm" disabled={issue.isPending} onClick={() => void create()}>
        {t(webhook.protected ? "webhooks.replaceToken" : "webhooks.createToken")}
      </Button>
      {webhook.protected ? (
        <Button type="button" variant="ghost" size="sm" onClick={() => setRemoving(true)}>
          {t("webhooks.removeToken")}
        </Button>
      ) : null}
      <TokenDialog token={token} address={webhook.address} variables={webhook.variables} onClose={() => setToken(null)} />
      <ConfirmDialog
        open={removing}
        onOpenChange={setRemoving}
        title={t("webhooks.removeTokenTitle")}
        description={t("webhooks.removeTokenDescription")}
        confirmLabel={t("webhooks.removeToken")}
        pending={remove.isPending}
        onConfirm={() => void drop()}
      />
    </div>
  );
}
