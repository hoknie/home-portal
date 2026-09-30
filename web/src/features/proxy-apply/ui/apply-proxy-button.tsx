"use client";

import { RefreshCw } from "lucide-react";
import { useTranslations } from "next-intl";
import { toast } from "sonner";

import { useCan } from "@/entities/session";
import { useApplyProxy } from "@/entities/proxy";
import { RequestError } from "@/shared/api";
import { Button } from "@/shared/ui/primitives";

export type ApplyProxyButtonProps = { disabled?: boolean };

function ApplyProxyButtonAllowed({ disabled = false }: ApplyProxyButtonProps) {
  const t = useTranslations("proxy");
  const apply = useApplyProxy();
  const run = async () => {
    try {
      await apply.mutateAsync();
      toast.success(t("applied"));
    } catch (error) {
      const message = error instanceof RequestError && error.message !== "" ? error.message : t("applyFailedUnknown");
      toast.error(t("applyFailed", { message }));
    }
  };
  return (
    <Button type="button" variant="outline" size="sm" disabled={disabled || apply.isPending} onClick={() => void run()}>
      <RefreshCw className={apply.isPending ? "animate-spin" : undefined} aria-hidden />
      {t("apply")}
    </Button>
  );
}

export function ApplyProxyButton(props: ApplyProxyButtonProps) {
  const can = useCan();
  return can("proxy", "update") ? <ApplyProxyButtonAllowed {...props} /> : null;
}
