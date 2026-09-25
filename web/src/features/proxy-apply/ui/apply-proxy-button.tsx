"use client";

import { RefreshCw } from "lucide-react";
import { useTranslations } from "next-intl";
import { toast } from "sonner";

import { useApplyProxy } from "@/entities/proxy";
import { RequestError } from "@/shared/api";
import { Button } from "@/shared/ui/primitives";

export type ApplyProxyButtonProps = { disabled?: boolean };

export function ApplyProxyButton({ disabled = false }: ApplyProxyButtonProps) {
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
